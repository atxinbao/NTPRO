# Rust Common Cache/Message Bus/Component Lifecycle Inventory

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-007

## Summary

This inventory covers Rust-only cutover gaps in:

- `crates/common/src/cache/**`
- `crates/common/src/msgbus/**`
- `crates/common/src/component.rs`
- dependent runtime lifecycle wiring in `crates/system`, `crates/live`,
  `crates/backtest`, `crates/data`, `crates/execution`, `crates/risk`, and
  `crates/portfolio`

Current state:

- `Cache` owns the in-memory market, execution, account, order, position,
  instrument, synthetic, quote/trade/bar, own-book, and snapshot state.
- `MessageBus` owns thread-local point-to-point, pub/sub, request/response,
  typed routing, Any-based routing, switchboard endpoint names, and event-store
  bus taps.
- `Component` owns the shared component state machine, global component
  registry, borrow guard, and lifecycle helpers used by actors, strategies,
  execution algorithms, and `Trader`.
- `NautilusKernel` wires cache, message bus, trader, data engine, execution
  engine, risk engine, portfolio, and optional event-store replay into one
  lifecycle.
- RCORE-007 does not authorize Python, PyO3, Cython, FFI, Cargo feature,
  cache persistence, message-bus routing, component lifecycle, or trading
  behavior removal. `removal_allowed = false`.

## Scope

In scope:

- Current Rust cache, message-bus, and component lifecycle modules.
- Runtime crates that depend on these modules for start/stop/dispose,
  cache hydration, message dispatch, and event-store capture.
- Existing Rust tests, ignored stress harnesses, and golden-trace gate blockers.
- Python/PyO3/FFI surfaces that remain active around this common runtime area.
- Follow-up routing for RCORE-008, RCORE-009, RTRACE, RREM, RADP, and release
  gate tasks.

Non-goals:

- No crate code changes.
- No public API changes.
- No message routing behavior changes.
- No cache persistence or snapshot behavior changes.
- No Python, PyO3, Cython, generated stub, or FFI removal.
- No adapter, venue, or external database behavior changes.

## Inventory Snapshot

Observed from local scans:

- Rust source files under `crates/common/src`: 112.
- Cache module files under `crates/common/src/cache`: 9.
- Message-bus module files under `crates/common/src/msgbus`: 11.
- Python-facing files observed across common/live/backtest wrappers relevant to
  this lifecycle area: 24.
- `crates/common/src/cache/tests.rs` contains broad in-memory cache coverage for
  reset/dispose, order and position indexes, quotes/trades/bars, own order
  books, account updates, position snapshots, and query APIs.
- `crates/common/src/msgbus/core.rs`, `api.rs`, `typed_router.rs`, and
  `typed_endpoints.rs` contain unit tests for routing, subscription ordering,
  handler removal, counters, endpoint dispatch, typed routes, Any routes, and
  bus tap behavior.
- `crates/common/src/component.rs` contains component state transition helpers,
  global component registry borrow tracking, and tests for borrow release after
  lifecycle calls and panics.
- `crates/live/tests/node.rs` covers Rust `LiveNode` handle and lifecycle
  control; `crates/live/tests/stress.rs` is an ignored stress harness for
  message-bus counters and live runner channels.
- `docs/rust-cutover/golden_trace/SCHEMA.md` defines the `cache_msgbus`
  category, but `docs/rust-cutover/golden_trace/GATE_EVIDENCE.md` records that
  no executable cache/message-bus ordering replay exists yet.

## Surface Matrix

| Area | Representative files | Current Rust status | Observed gap |
| --- | --- | --- | --- |
| Cache in-memory state | `crates/common/src/cache/mod.rs`, `cache/index.rs`, `cache/refs.rs`, `cache/tests.rs` | `Cache` stores core runtime state and has broad local unit coverage for reset, dispose, filtering, indexes, snapshots, and order/position/account updates. | Local tests are not yet organized into a release-gate matrix for cache update ordering, replay ordering, and cache/msgbus event sequencing. |
| Cache database backing | `crates/common/src/cache/database.rs`, `cache/mod.rs` | `CacheDatabaseAdapter` defines load/add/update/delete/snapshot operations; `Cache::set_database`, `cache_all`, `cache_*`, `flush_db`, and `dispose` call the adapter when configured. | The database module is marked under development; some persistence paths are intentionally no-op or incomplete, and no Redis or durable backing fixture is release-gated here. |
| Cache snapshots | `crates/common/src/cache/mod.rs`, `cache/tests.rs` | In-memory position snapshot frames are covered by tests; `CacheSnapshotRef` and blob restore helpers exist. | `snapshot_position_state` still ends in `todo!()` after the optional database call, and order snapshot persistence is commented out in the order update path. |
| Message bus routing | `crates/common/src/msgbus/core.rs`, `api.rs`, `typed_router.rs`, `typed_endpoints.rs` | Message bus supports typed routing, Any routing, endpoint sends, response handlers, subscriptions, priority ordering, and route caches. | Typed and Any routes are intentionally separate. The final Rust-only contract needs explicit tests and docs proving no silent route mismatch in product/runtime paths. |
| Message bus global state | `crates/common/src/msgbus/mod.rs`, `crates/system/src/kernel.rs`, `crates/data/tests/engine.rs` | The bus is thread-local. `NautilusKernel::new_with` installs a per-kernel message bus, and test fixtures can register a fresh bus. | Thread-local global state still requires explicit isolation evidence for repeated kernels, live tests, backtest tests, and replay harnesses. |
| Message bus backing database | `crates/common/src/msgbus/database.rs`, `crates/common/src/msgbus/core.rs` | `MessageBusConfig` and `MessageBusDatabaseAdapter` define database-facing configuration and facade traits. | `MessageBus::close` still documents a TODO for backing database integration, and `has_backing` is initialized false without a Rust release-backed adapter path in this scope. |
| Bus tap and golden trace capture | `crates/common/src/msgbus/mod.rs`, `api.rs`, `crates/common/src/timer.rs`, `crates/system/src/kernel.rs` | Bus taps capture publish, send, response, and time-event dispatch before handlers observe messages; tests cover clearing and reentrant tap replacement. | Golden trace gate still lacks executable `cache_msgbus` ordering replay, so tap existence alone is not release evidence. |
| Component state machine | `crates/common/src/component.rs`, `crates/common/src/enums.rs` | `ComponentState` and `ComponentTrigger` define start/stop/resume/reset/dispose/degrade/fault transitions. | State-machine coverage exists locally, but RCORE-008 should pin a public test matrix for invalid transitions, panic release, and runtime component ordering. |
| Component registry | `crates/common/src/component.rs`, `crates/system/src/trader.rs` | Components register globally with `Rc<UnsafeCell<dyn Component>>`; borrow tracking prevents simultaneous mutable lifecycle access. | The registry is thread-local/global by design and uses `UnsafeCell`; release readiness needs targeted justification and lifecycle isolation tests beyond unit borrow checks. |
| Trader and kernel lifecycle | `crates/system/src/trader.rs`, `crates/system/src/kernel.rs`, `crates/backtest/src/engine.rs`, `crates/live/src/node.rs` | `Trader` starts/stops/resets/disposes actors, strategies, and execution algorithms. `Kernel` starts engines, initializes trader, finalizes stop, disposes engines/cache/msgbus, and seals event-store runs. | Cross-component ordering is distributed across runtime crates. Final closure needs explicit smoke/golden evidence that cache, msgbus, trader, engines, and event-store shutdown order remains deterministic. |
| Live runtime stress | `crates/live/tests/node.rs`, `crates/live/tests/stress.rs` | Live node integration tests cover handle control, shutdown commands, and lifecycle states; stress harness measures message-bus counter deltas and cancel starvation. | Stress tests are ignored by default and not part of the required fast verification gate; RCORE-008 must decide which deterministic subset belongs in default or release verification. |
| Python/PyO3 wrappers | `crates/common/src/python/cache.rs`, `crates/common/src/python/msgbus.rs`, `crates/live/src/python/node.rs`, `crates/backtest/src/python/*.rs` | Python wrappers expose cache, message bus, config, node, engine, clock, timer, and enum surfaces over Rust internals. | Python/PyO3 remains active and cannot be removed in this RCORE task. Rust-only removal still requires a dedicated RREM gate and scope decision. |

## Gap Register

| ID | Gap | Evidence | Impact | Follow-up |
| --- | --- | --- | --- | --- |
| CML-001 | `cache_msgbus` golden trace category exists without executable replay. | `docs/rust-cutover/golden_trace/SCHEMA.md` lists `cache_msgbus`; `GATE_EVIDENCE.md` says no executable cache/message-bus ordering replay exists yet. | Final Rust-only release cannot claim deterministic cache/message-bus replay parity. | RCORE-008 should add focused Rust tests; RTRACE should bind an executable `cache_msgbus` replay. |
| CML-002 | Cache database backing is not release-gated. | `cache/database.rs` is marked under development; `CacheDatabaseAdapter` is a trait surface, while local tests mostly use in-memory or mock behavior. | Rust-first runtime can use in-memory cache, but durable cache backing cannot be treated as release-ready. | RCORE-008 mock adapter tests; NDB/RADP fixture strategy for real backing stores. |
| CML-003 | Cache snapshot persistence is incomplete. | `Cache::snapshot_position_state` calls the database when present and then `todo!()`; order snapshot persistence is commented out after `database.add_order`. | Snapshot-backed recovery and replay cannot rely on this path for release closure. | RCORE-008 should add explicit failing/blocked tests or scoped assertions; RCORE-009 should close or defer with gatekeeper approval. |
| CML-004 | Message-bus database integration is a facade, not a proven runtime path. | `MessageBusConfig` and `MessageBusDatabaseAdapter` exist, but `MessageBus::close` has `TODO: Integrate the backing database`, and `MessageBus::new` starts with `has_backing: false`. | External/pubsub persistence claims remain blocked. | RCORE-009 or NDB task must either implement, defer, or remove the release claim after scope approval. |
| CML-005 | Typed and Any message-bus routes can diverge silently when publishers and subscribers use mismatched APIs. | `msgbus/core.rs` documents that typed and Any routes are separate and mixing them causes silent message loss. | Rust product/runtime entrypoints need evidence that hot-path data and Python/custom callbacks subscribe through matching APIs. | RCORE-008 route mismatch tests; RCORE-009 docs or helper guardrails if needed. |
| CML-006 | Thread-local/global message bus and component registries need lifecycle isolation evidence. | `msgbus/mod.rs` uses thread-local `MESSAGE_BUS`; `component.rs` uses thread-local `COMPONENT_REGISTRY`; `Kernel::new_with` installs a message bus per kernel. | Repeated backtest/live runs may leak handlers/components if lifecycle cleanup is incomplete. | RCORE-008 repeated-kernel and repeated-component lifecycle tests. |
| CML-007 | Component registry uses `UnsafeCell` behind borrow tracking. | `component.rs` registers `Rc<UnsafeCell<dyn Component>>`; borrow guard tests cover release after lifecycle call and panic. | Existing unit tests reduce immediate risk, but release gate still needs a focused unsafe/lifecycle justification. | Verification & Release Gatekeeper unsafe/lifecycle audit; RCORE-008 targeted tests. |
| CML-008 | Kernel shutdown ordering is not captured as a cache/msgbus golden trace. | `Kernel::dispose` stops engines, resets portfolio, cancels timers, seals event store, disposes engines, cache, and message bus. | Ordering regressions could affect residual events, cache final state, and event-store replay. | RCORE-008 deterministic smoke; RTRACE `cache_msgbus` trace harness. |
| CML-009 | Live stress evidence is not part of default verification. | `crates/live/tests/stress.rs` is ignored and release-only by instruction, while default fast verification does not run ignored stress scenarios. | Throughput/starvation behavior remains useful evidence but not a default PR gate. | RCORE-008 should choose a deterministic small smoke and leave heavy stress to release/perf verification. |
| CML-010 | Python/PyO3 wrappers remain active across cache, message bus, clock/timer, and live/backtest node surfaces. | `crates/common/src/python/cache.rs`, `msgbus.rs`, `clock.rs`, `timer.rs`, `runtime.rs`, `crates/live/src/python/node.rs`, and `crates/backtest/src/python/*.rs` are present. | Rust-only removal remains blocked until product surface, runtime, adapter, QA, and RREM gates approve it. | RREM tasks only after Rust-only route and removal gates are approved. |

## Existing Test Surface

Observed local Rust coverage:

- `crates/common/src/cache/tests.rs`
  - reset/dispose/flush/general cache behavior;
  - order and position indexing;
  - quote/trade/bar/funding/instrument-status storage;
  - account state updates;
  - own order book lifecycle;
  - position snapshot roundtrip and filtering;
  - query API consistency.
- `crates/common/src/msgbus/core.rs`
  - endpoint registration and sends;
  - pub/sub counters;
  - matching subscriptions and priority ordering;
  - late wildcard subscription cache backfill.
- `crates/common/src/msgbus/api.rs`
  - typed endpoint dispatch;
  - typed unsubscribe behavior;
  - reentrant account-state topic access;
  - bus tap publish/send/response capture;
  - clearing and reentrant replacement of bus taps.
- `crates/common/src/component.rs`
  - component registry borrow tracking;
  - borrow release after lifecycle calls;
  - borrow release after panic.
- `crates/system/src/trader.rs`
  - trader/component lifecycle;
  - restrictions for adding components while running or disposed;
  - clear actor/strategy/execution-algorithm paths.
- `crates/live/tests/node.rs`
  - `LiveNode` handle state, start/stop errors, graceful shutdown by handle and
    shutdown-system topic, and maintenance dispatcher behavior.
- `crates/live/tests/stress.rs`
  - ignored release/stress scenarios for trade bursts and cancel starvation.
- `crates/data/tests/engine.rs`
  - a fresh registered message bus fixture and broad data-engine message
    routing/cache interaction tests.

RCORE-007 does not add tests. RCORE-008 should turn the gap register above into
a targeted test matrix and explicitly decide which heavy stress checks stay
release-only.

## RCORE-008 Test Matrix

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-008

RCORE-008 adds focused Rust unit coverage without changing runtime behavior.

| Gap | Test coverage added | Scope decision |
| --- | --- | --- |
| CML-003 | `cache::tests::test_snapshot_position_state_release_blocker_is_explicit` | Keeps the current `snapshot_position_state` `todo!()` visible as a release blocker instead of silently treating snapshot persistence as done. |
| CML-005 | `msgbus::api::tests::test_route_separation_any_subscriber_does_not_receive_typed_quote` and `test_route_separation_typed_subscriber_does_not_receive_any_quote` | Pins the typed-vs-Any route split as explicit behavior. Product/runtime publishers and subscribers must choose matching APIs. |
| CML-006 | `msgbus::api::tests::test_message_bus_thread_local_isolation_for_lifecycle_state` and `component::tests::test_component_registry_is_thread_local_for_lifecycle_isolation` | Confirms thread-local lifecycle state does not leak message-bus subscriptions or component registry entries across worker threads. |
| CML-007 | `component::tests::test_component_registry_is_thread_local_for_lifecycle_isolation` complements existing borrow-release and panic-release tests. | UnsafeCell registry remains in place; this task adds isolation evidence only and does not approve unsafe/lifecycle release closure. |

Not closed by RCORE-008:

- CML-001 and CML-008 still require executable `cache_msgbus` golden trace replay.
- CML-002 and CML-004 still require cache/message-bus backing database fixture or explicit deferral.
- CML-009 remains release/performance verification, not a default PR gate.
- CML-010 remains blocked by Rust-only removal gates; no Python/PyO3/Cython files were changed.

## Release Gate Decision

RCORE-007 is an inventory task only.

- `removal_allowed = false`
- No Python/PyO3/Cython/FFI deletion is allowed here.
- No cache, message-bus, component lifecycle, engine lifecycle, event-store,
  database, or adapter behavior changes are allowed here.
- RCORE-008 should add targeted Rust tests for the highest-risk gaps.
- RCORE-009 should either close the gaps or record explicit scope deferrals.
- This decision does not mark the Rust-only release gate as passed. It records
  the current common-runtime gap register that later tasks must close or defer.
