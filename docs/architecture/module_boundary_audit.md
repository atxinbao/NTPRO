# NTPRO Module Boundary Audit

Date: 2026-06-04
Executor: Codex
Task ID: NARCH-006

## Purpose

This audit checks the current Rust-only module boundaries before any refactor,
dashboard-facing state extraction, observability contract, or control API work.

It is documentation-only. It does not refactor crates, change runtime behavior,
add dashboard implementation, or add live control endpoints.

## Workspace Shape

`cargo metadata --no-deps --format-version=1` reports 41 workspace packages.

The current crate layout broadly matches the architecture map:

| Boundary | Primary crates | Current assessment |
| --- | --- | --- |
| Product surface | `nautilus-cli` | Clear. CLI owns user command surface. Runtime wiring is still partial for some commands. |
| Backtest runtime | `nautilus-backtest` | Mostly clear. Owns backtest engine, exchange simulation, execution client, node, config, and results. |
| Live/sandbox runtime | `nautilus-live`, `nautilus-sandbox` | Mostly clear. `nautilus-live` owns node lifecycle; `nautilus-sandbox` owns simulated adapter behavior. |
| System composition | `nautilus-system` | Clear owner for kernel/trader composition, but it naturally touches many runtime modules. |
| Shared runtime infrastructure | `nautilus-common` | Mixed. It contains cache, message bus, clocks, logging, actor helpers, factories, live runner support, and shared messages. |
| Domain model | `nautilus-model`, `nautilus-core` | Clear. Model owns trading domain types; core owns shared primitives. |
| Data engine | `nautilus-data` | Mostly clear. It owns subscriptions, requests, aggregation, and data client paths. |
| Execution engine | `nautilus-execution` | Mixed but intentional. It owns execution engine, matching engine, order emulator, order manager, reconciliation, and execution models. |
| Risk engine | `nautilus-risk` | Clear. Owns risk checks and risk configuration. |
| Portfolio | `nautilus-portfolio` | Clear. Owns account state, exposure, PnL, and snapshots. |
| Persistence and event store | `nautilus-persistence`, `nautilus-event-store`, `nautilus-serialization` | Partially split. Persistence boundary needs a follow-up contract before replay/dashboard work consumes artifacts. |
| Network/infrastructure | `nautilus-network`, `nautilus-infrastructure`, `nautilus-cryptography`, `nautilus-plugin` | Clear enough for current use. Dashboard/control should not depend on these internals directly. |
| Adapter layer | `crates/adapters/*` | Clear outer boundary. Individual adapter evidence varies by venue. |
| Verification support | `nautilus-testkit`, `tests/golden`, `scripts/ai` | Clear. Local verification remains outside runtime product APIs. |

## Mixed Concerns

The following areas are not immediate problems, but they should be treated
carefully before refactor or dashboard work.

| Area | Mixed concern | Risk if consumed directly | Follow-up |
| --- | --- | --- | --- |
| `nautilus-common` | Cache, message bus, clock, logging, live helpers, factories, and shared actor support live together. | Dashboard or control code could couple to internal cache/message-bus details instead of a stable status model. | NARCH-002 should define common/cache/msgbus contracts; NARCH-004 should define observability state. |
| `nautilus-execution` | Execution engine, matching engine, order emulator, reconciliation, and order manager share one crate. | Future UI/control code could bypass RiskEngine or order lifecycle contracts by calling lower-level pieces directly. | NARCH-002 should define execution input/output boundaries; NARCH-005 should define allowed control actions. |
| `nautilus-live` | Node builder, node, manager, runner, and emitter live in one runtime crate. | Control work could mix lifecycle state, command handling, and external endpoint behavior. | NARCH-003 should define lifecycle state machine before any control API. |
| `nautilus-system` | Kernel/trader composition sees many components and registries. | Dashboard code could read component registries or actor internals directly. | NARCH-002 and NARCH-004 should define stable read-only status output. |
| Persistence/event-store/serialization | Artifact ownership is split across three crates. | Trace replay or dashboard history could depend on unstable artifact details. | Later persistence and trace tasks should define stable artifact contracts. |
| Adapter crates | Adapter crates often contain HTTP/WebSocket clients, parsers, examples, bins, benches, and tests. | Product docs could over-claim support or require real credentials. | Keep using `docs/integrations/adapter_support_matrix.md` and future fixture manifests. |

## Dashboard Must Not Read Directly

Future dashboard or operator surfaces must not read these internals directly:

| Internal detail | Why not |
| --- | --- |
| `Cache` mutable internals and borrow wrappers | Cache mutation is runtime-owned and must not become a UI data API. |
| `MessageBus` routing tables, stubs, switchboards, or typed endpoint internals | Message routing is a transport/control detail, not a dashboard state contract. |
| `Trader` component and actor registries | Registries are composition internals and can change with runtime wiring. |
| `ExecutionEngine` order manager, matching engine, emulator, and reconciliation internals | Bypassing the engine contract risks inconsistent order lifecycle and risk semantics. |
| `RiskEngine` internal checks and bypass flags | Dashboard should report risk state, not invoke or mutate checks directly. |
| `PortfolioState` internals | Dashboard should consume summary/snapshot models, not internal accounting state. |
| Adapter clients and credential/config internals | Avoid credential leakage, live endpoint coupling, and venue-specific assumptions. |
| Persistence database handles and event-store codec internals | Dashboard/history APIs need stable artifact contracts first. |
| Verification scripts and Shrimp/agentflow state | These are repository control tools, not product runtime state. |

## Candidate Stable Telemetry Surface

The following state should later be exposed through stable read-only contracts,
not by reading engine internals:

| Surface | Candidate fields |
| --- | --- |
| Node status | node id, environment, lifecycle state, start time, stop reason, error summary. |
| Data status | connected data clients, subscribed instruments, last event time, data lag, dropped/error counts. |
| Execution status | connected execution clients, order counts by state, last report time, reconciliation status. |
| Risk status | trading enabled/disabled, rejection counts by reason, configured limits summary. |
| Portfolio summary | account ids, cash/equity summary, open position count, realized/unrealized PnL summary. |
| Adapter status | adapter id, status, fixture/sandbox/live classification, last heartbeat, credential mode without secret values. |
| Cache summary | counts by entity type, last update time, snapshot availability. |
| Verification status | release source point, last local verification command, result summary. |

These fields belong to NARCH-004 and later implementation tasks. NARCH-006 only
records the boundary candidates.

## Refactor Candidates

These are candidates only. They are not executable changes in this task.

| Candidate | Rationale | First safe next step |
| --- | --- | --- |
| Common cache/message-bus contract split | `nautilus-common` is broad and central. | Document contracts before any crate split. |
| Execution sub-boundary contract | Matching, emulation, reconciliation, and routing have different consumers. | Document inputs/outputs and which surfaces are public. |
| Live node lifecycle contract | Control work needs stable lifecycle states. | Complete NARCH-003 before adding control endpoints. |
| Persistence artifact contract | Trace/replay/dashboard history needs stable artifacts. | Record event-store/cache/message-bus artifact boundary. |
| Adapter fixture manifests | Support matrix needs compact evidence per supported adapter. | Add per-adapter manifest tasks before broad trace expansion. |

## Current Boundary Verdict

The current crate layout is good enough for v0.2.0 product hardening, CLI
work, examples, adapter support documentation, and trace planning.

It is not yet safe for dashboard/control implementation to consume runtime
internals directly. The next architecture tasks should define module contracts,
node lifecycle, observability state, and control actions before implementation
work starts.
