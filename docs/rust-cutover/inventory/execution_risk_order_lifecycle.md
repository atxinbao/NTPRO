# Rust Execution/Risk/Order Lifecycle Inventory

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-010

## Summary

This inventory covers Rust-only cutover gaps in:

- `crates/execution`
- `crates/risk`
- order lifecycle behavior crossing execution, risk, model, cache, portfolio,
  and golden-trace gates

Current state:

- `nautilus-execution` owns Rust execution routing, execution clients, order
  manager, matching engine, order emulator, reconciliation, and order/position
  event processing.
- `nautilus-risk` owns Rust pre-trade command gating, submit/modify throttlers,
  notional and balance checks, trading-state checks, and denial/rejection event
  emission.
- Both crates have substantial Rust tests, but the final Rust-only release gate
  still needs executable evidence for risk decisions, execution routing, order
  lifecycle replay, and position open/increase/reduce/close behavior.
- RCORE-010 is an inventory task only. It does not authorize Python, PyO3,
  Cython, Cargo feature, adapter, execution, risk, order, position, portfolio,
  or public API removal. `removal_allowed = false`.

## Scope

In scope:

- Current Rust modules, feature flags, tests, TODOs, panics, parity comments,
  ignored tests, and golden-trace blockers in execution/risk/order lifecycle
  paths.
- Rust routing from `RiskEngine` to `ExecutionEngine`.
- Order manager, matching engine, order emulator, and reconciliation boundaries.
- Existing golden trace categories and release gate blockers relevant to risk,
  execution, order lifecycle, positions, and portfolio/PnL.
- Python/PyO3/Cython surfaces that remain active around execution and risk.
- Follow-up routing for RCORE-011, RCORE-012, RTRACE, RADP, RREM, and release
  gate tasks.

Non-goals:

- No crate code changes.
- No public API changes.
- No trading semantic changes.
- No new tests in this inventory task.
- No Python, PyO3, Cython, generated stub, or FFI removal.
- No adapter, venue, exchange, account, portfolio, or persistence behavior
  changes.

## Inventory Snapshot

Observed from local scans:

- Rust source files under `crates/execution/src`: 39.
- Rust source files under `crates/risk/src`: 6.
- Rust integration test files under `crates/execution/tests`: 3.
- Rust integration test files under `crates/risk/tests`: 1.
- Test markers observed in main integration test files:
  - `crates/execution/tests/exec_engine.rs`: 312.
  - `crates/execution/tests/matching_engine.rs`: 297.
  - `crates/execution/tests/order_emulator.rs`: 2.
  - `crates/risk/tests/risk_engine.rs`: 168.
- `ExecutionEngine::register_msgbus_handlers` registers direct and queued
  command endpoints plus order-event and execution-report endpoints.
- `RiskEngine::register_msgbus_handlers` registers direct and queued risk
  command endpoints; submit and modify throttler success handlers forward to
  `exec_engine_queue_execute`.
- `tests/golden/order_lifecycle_schema.jsonl` contains six
  `order_lifecycle` rows, but current gate evidence records this as schema-only
  seed evidence rather than a full Rust execution/risk replay.
- `docs/rust-cutover/golden_trace/GATE_EVIDENCE.md` records release blockers for
  `risk`, `execution`, `position`, and `portfolio_pnl`.
- `crates/execution` and `crates/risk` both still expose `python` and
  `extension-module` feature surfaces through PyO3.

## Surface Matrix

| Area | Representative files | Current Rust status | Observed gap |
| --- | --- | --- | --- |
| Execution command routing | `crates/execution/src/engine/mod.rs`, `crates/common/src/msgbus/switchboard.rs` | `ExecutionEngine` accepts `TradingCommand`, queued commands, order events, and execution reports through typed message-bus endpoints. | Routing exists, but final release evidence needs a deterministic trace that includes risk-to-execution queueing, client selection, order events, and cache effects. |
| Risk command routing | `crates/risk/src/engine/mod.rs`, `crates/risk/tests/risk_engine.rs` | `RiskEngine` accepts direct and queued commands and forwards accepted submit/modify commands to `exec_engine_queue_execute`. | Risk accept/reject decisions are locally tested, but no executable golden replay covers rate limits, notional checks, trading-state gates, and denial event ordering. |
| Submit/modify throttlers | `crates/risk/src/engine/mod.rs` | Submit and modify throttlers create denied or rejected events on failure and forward accepted commands to execution. | Throttler behavior needs release-gate evidence across both success and denial paths, including event ordering through `ExecutionEngine.process`. |
| Balance and notional checks | `crates/risk/src/engine/mod.rs`, `crates/risk/tests/risk_engine.rs` | Risk checks cash, margin, betting, free balance, pending orders, reducing orders, max notional, and trading state. | Several risk tests are ignored or TODO-scoped, including order-list reducing checks, high-precision work, emulator integration, and real-time account balance tracking. |
| Order manager lifecycle | `crates/execution/src/order_manager/manager.rs`, `crates/execution/tests/exec_engine.rs` | Order manager handles local order state, submit command cache, OTO/OCO/OUO contingencies, cancel, modify, and child order submission. | Some contingency paths still panic on missing cached orders. Follow-up tests need to decide whether these are invariant-only panics or release blockers. |
| Matching engine | `crates/execution/src/matching_engine/engine.rs`, `crates/execution/tests/matching_engine.rs` | Matching engine handles simulated matching, order release, trade/quote/bar processing, fill generation, contingent orders, and Cython parity behavior. | `PriceType::Mark` is not implemented; clock fixed-time setting remains TODO; several contingent/stale-local tests are ignored. |
| Order emulator | `crates/execution/src/order_emulator/emulator.rs`, `crates/execution/tests/order_emulator.rs` | Emulator queues and releases emulated stop/limit orders, emits risk events, and waits when market data is unavailable. | Trigger/transform path still has a TODO, invalid stop-order type panics, and risk tests say emulator integration is not yet enabled. |
| Reconciliation | `crates/execution/src/reconciliation/**` | Reconciliation has broad tests and property-style coverage for order and position report adjustment. | It explicitly mirrors Python behavior in fill-quantity mismatch handling and documents a limitation where an `OrderUpdated` alone does not transition `PartiallyFilled` to `Filled`. |
| Position lifecycle | `crates/execution/src/engine/mod.rs`, `crates/portfolio/**`, `crates/model/**` | Execution engine assigns hedging/netting position IDs and handles open/update/flip flows after fills. | Position open/increase/reduce/close/flip has no executable golden trace yet; some implementation comments still reference Python behavior. |
| Golden trace gate | `tests/golden/order_lifecycle_schema.jsonl`, `docs/rust-cutover/golden_trace/GATE_EVIDENCE.md` | Schema rows exist for submit accept/reject, modify, cancel, triggered fill, and partial-to-filled lifecycle cases. | Current gate evidence marks `order_lifecycle` as schema-only; `risk`, `execution`, `position`, and `portfolio_pnl` remain release blockers. |
| Python/PyO3/Cython surfaces | `crates/execution/Cargo.toml`, `crates/risk/Cargo.toml`, `src/python/**`, `matching_engine/config.rs` | `python` and `extension-module` features remain active; execution matching config still locks Rust defaults to a Cython constructor contract. | Rust-only removal remains blocked until product surface, runtime, adapter, QA, and RREM gates approve it. |

## Gap Register

| ID | Gap | Evidence | Impact | Follow-up |
| --- | --- | --- | --- | --- |
| EROL-001 | `order_lifecycle` has schema seed rows but no full Rust execution/risk replay. | `tests/golden/order_lifecycle_schema.jsonl` has six cases; `GATE_EVIDENCE.md` lists it as schema-only seed evidence. | Release cannot claim end-to-end Rust order lifecycle parity from schema validation alone. | RCORE-011 should add or explicitly scope executable Rust order lifecycle tests; RTRACE should bind golden replay when ready. |
| EROL-002 | Risk accept/reject decisions are not executable golden-trace evidence. | `GATE_EVIDENCE.md` says `risk` has no executable Rust golden trace for accept/reject, rate limits, notional checks, or trading-state gates. | Risk regression could pass schema checks while changing live/backtest denial behavior. | RCORE-011 risk test matrix; RCORE-012 closure or scoped deferral; RTRACE risk replay. |
| EROL-003 | Risk-to-execution queued routing is not release-gated end to end. | `RiskEngine` forwards accepted submit/modify commands to `exec_engine_queue_execute`; `ExecutionEngine` receives queued/direct commands and events. | Reentrancy-safe queueing can regress without a trace spanning both engines. | RCORE-011 deterministic routing smoke from risk command to execution event; RCORE-012 closure. |
| EROL-004 | Order manager contingency panics need release classification. | OTO/OCO/OUO paths panic when expected cached child or contingent orders are missing. | Invariant panics may be acceptable, but release gate needs evidence distinguishing impossible states from recoverable venue/runtime states. | RCORE-011 targeted tests or explicit invariant decision; RCORE-012 closure. |
| EROL-005 | Matching engine has known unimplemented/deferred branches. | `PriceType::Mark` panics as not implemented; `iterate` still has a TODO for fixed clock time; matching tests include ignored contingent-order stale-local cases. | Simulated execution parity is incomplete for those paths. | RCORE-011 should pin current behavior; RCORE-012 should implement or scope deferrals. |
| EROL-006 | Order emulator integration with risk is not fully closed. | `order_emulator` has a trigger transform TODO; risk tests have ignored emulator-integration cases. | Emulated order behavior cannot be treated as release-ready for all risk/execution paths. | RCORE-011 emulator/risk test decision; RCORE-012 closure or gate deferral. |
| EROL-007 | Reconciliation retains Python-parity limitations. | Reconciliation code mirrors Python fill-mismatch behavior; tests document the `OrderUpdated` limitation shared with Python reference. | Rust-only release needs a decision on whether to preserve, fix, or explicitly defer the limitation. | RCORE-011 regression coverage; RCORE-012 scope decision. |
| EROL-008 | Position lifecycle lacks executable release trace. | `ExecutionEngine` handles hedging/netting IDs and flip logic, but `GATE_EVIDENCE.md` says no executable position open/increase/reduce/close trace exists. | Position/PnL regressions could pass order-only tests. | RCORE-011 position lifecycle smoke; RTRACE position trace; portfolio/PnL gate tasks. |
| EROL-009 | Portfolio/PnL/account balance release gate is outside current proof. | `GATE_EVIDENCE.md` records no executable account balance, margin, realized PnL, unrealized PnL, or equity replay. Risk tests also defer real-time account balance tracking. | Execution/risk closure cannot imply portfolio/PnL release readiness. | RCORE/RTRACE/portfolio follow-up before release gate. |
| EROL-010 | Python/PyO3/Cython surfaces remain active around execution and risk. | `Cargo.toml` feature flags, `src/python/**`, and Cython parity comments/defaults remain present. | Rust-only removal is blocked. | RREM only after Rust-only route, runtime, adapter, QA, and release gates approve removal. |

## Existing Test Surface

Observed local Rust coverage:

- `crates/execution/tests/exec_engine.rs`
  - execution engine command and event processing;
  - order accepted/rejected/denied/fill scenarios;
  - position ID assignment and execution-client interactions;
  - reconciliation and external-order scenarios.
- `crates/execution/tests/matching_engine.rs`
  - simulated matching, trade/quote/bar processing, limit/market order fills;
  - contingent orders, trigger handling, latency/fee/fill model behavior;
  - some ignored stale-local contingent-order cases.
- `crates/execution/tests/order_emulator.rs`
  - current minimal emulator integration smoke.
- `crates/risk/tests/risk_engine.rs`
  - submit/modify throttling, invalid order validation, notional/balance checks,
    trading-state gates, and denial/rejection event flows;
  - ignored/TODO cases for order-list reduction, high precision, emulator
    integration, and real-time account balance tracking.
- `crates/execution/src/reconciliation/tests.rs`
  - broad order and position report reconciliation behavior, including
    documented Python-parity limitations.
- `tests/golden/order_lifecycle_schema.jsonl`
  - six schema-level lifecycle cases for submit, reject, modify, cancel,
    triggered fill, and partial-to-filled flows.

RCORE-010 does not add tests. RCORE-011 should convert the gap register above
into a targeted execution/risk/order lifecycle test matrix and identify any
remaining blockers that must stay scoped rather than treated as done.

## RCORE-011 Test Matrix

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-011

RCORE-011 adds focused Rust regression assertions without changing runtime
behavior.

| Gap | Test coverage added or identified | Scope decision |
| --- | --- | --- |
| EROL-002 | Strengthened `crates/risk/tests/risk_engine.rs::test_submit_order_with_default_settings_then_sends_to_client` to assert an accepted submit command forwards to `exec_engine_queue_execute` and emits no `ExecEngine.process` denial/rejection event. Strengthened `test_submit_order_when_trading_halted_then_denies_order` to assert a halted submit emits `OrderDenied` and forwards nothing to execution. | Risk accept/deny event routing now has focused Rust assertions. Full risk golden replay remains RTRACE/RCORE-012 scope. |
| EROL-003 | Same two risk tests pin the bidirectional routing split: accepted risk commands use execution queue, denied commands stay on process events only. | Covers the RiskEngine side of risk-to-execution routing. A broader trace that includes an actual `ExecutionEngine` receiving and processing the queued command remains RCORE-012/RTRACE scope. |
| EROL-001 | Existing `tests/golden/order_lifecycle_schema.jsonl` keeps six schema rows for submit, reject, modify, cancel, triggered fill, and partial-to-filled cases. | Still schema-only. No full execution/risk replay is claimed by RCORE-011. |
| EROL-004 | Existing execution tests cover many order manager paths; contingency missing-cache panic classification remains open. | Not closed by RCORE-011. RCORE-012 must classify invariant panics or add targeted tests. |
| EROL-005 | Existing matching-engine tests remain in place; ignored stale-local cases and `PriceType::Mark` are still not closed. | Not closed by RCORE-011. Matching-engine implementation/scope decisions remain RCORE-012 work. |
| EROL-006 | Existing order-emulator smoke remains minimal and risk tests still defer emulator integration. | Not closed by RCORE-011. Emulator/risk integration remains scoped blocker evidence. |
| EROL-007 | Existing reconciliation tests document Python-parity limitations. | Not closed by RCORE-011. RCORE-012 must decide preserve/fix/defer. |
| EROL-008 | Existing execution tests cover opening, increasing, and closing netting positions. | Helpful existing coverage, but no executable golden position trace is claimed. |
| EROL-009 | Existing risk tests include account/balance checks, but real-time account balance tracking and portfolio/PnL replay remain deferred. | Not closed by RCORE-011. Portfolio/PnL release evidence remains later gate work. |
| EROL-010 | RCORE-011 changes no Python/PyO3/Cython files. | Removal remains blocked by RREM/release gates. |

## RCORE-012 Closeout Matrix

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-012

RCORE-012 closes the execution/risk/order lifecycle inventory by separating
Rust evidence that is safe to claim now from release-gate work that must remain
explicitly scoped. It does not change runtime behavior, trading semantics,
public APIs, adapters, persistence formats, or Python/PyO3/Cython surfaces.

| Gap | RCORE-012 decision | Evidence | Follow-up owner |
| --- | --- | --- | --- |
| EROL-001 | Explicitly scoped, not closed as replay parity. | `tests/golden/order_lifecycle_schema.jsonl` still contains six schema rows, and `docs/rust-cutover/golden_trace/GATE_EVIDENCE.md` still labels `order_lifecycle` as schema-only seed evidence. | RTRACE must bind these rows to an executable Rust order lifecycle replay before release. |
| EROL-002 | Locally closed for focused Rust risk accept/deny assertions; release replay remains scoped. | RCORE-011 strengthened `crates/risk/tests/risk_engine.rs::test_submit_order_with_default_settings_then_sends_to_client` and `test_submit_order_when_trading_halted_then_denies_order`; `cargo test -p nautilus-risk --test risk_engine` passed in RCORE-011 evidence. | RTRACE still owns executable risk golden replay for rate limits, notional checks, and trading-state ordering. |
| EROL-003 | Locally closed for RiskEngine-side queue routing; end-to-end trace remains scoped. | RCORE-011 asserted accepted submits forward through `exec_engine_queue_execute` and halted denials do not forward to execution. | RTRACE/RCORE follow-up must replay a queued command through an actual `ExecutionEngine` and compare emitted events. |
| EROL-004 | Explicitly scoped as an order-manager invariant, not silently treated as release-ready recovery behavior. | `crates/execution/src/order_manager/manager.rs` documents panic paths for missing contingent orders. Existing execution tests cover incomplete order-list denial and OTO child `position_id` propagation. Missing cached contingent children remain an internal invariant decision, not a venue/runtime recovery contract. | A future RCORE task must either convert these invariant panics into recoverable errors with migration notes and tests, or keep them as documented release exceptions approved by the gatekeeper. |
| EROL-005 | Explicitly scoped. | `PriceType::Mark` still panics as not implemented, the matching engine still has a fixed-clock TODO, and ignored contingent stale-local tests remain present in `crates/execution/tests/matching_engine.rs`. | RCORE matching-engine task must implement or formally defer `PriceType::Mark`, fixed-clock semantics, and stale contingent-order behavior before final release. |
| EROL-006 | Explicitly scoped. | Risk tests still ignore emulator-routing cases with “Waiting on emulator implementation”; order emulator smoke coverage exists but does not prove RiskEngine integration. | RCORE/RTRACE must add emulator-to-risk executable coverage or record a release-scope exclusion. |
| EROL-007 | Explicitly scoped as preserved Python-parity limitation. | Reconciliation tests document the shared Python limitation where `OrderUpdated` alone does not transition `PartiallyFilled` to `Filled`. | RCORE or release gate must decide preserve/fix/defer before Rust-only release; any behavior change needs migration notes and tests. |
| EROL-008 | Partially covered by existing Rust execution tests, but release trace remains scoped. | Execution tests cover position ID assignment, open/increase/close flows, reduce-only fills, and OTO child position linking. `GATE_EVIDENCE.md` still records no executable position open/increase/reduce/close trace replay. | RTRACE must add executable position lifecycle replay before release. |
| EROL-009 | Explicitly scoped. | `GATE_EVIDENCE.md` still records no executable account balance, margin, realized PnL, unrealized PnL, or equity replay; risk tests still ignore account balance tracking. | Portfolio/PnL runtime and trace tasks must close this before release. |
| EROL-010 | Explicitly blocked. | `Cargo.toml`, `pyproject.toml`, `build.py`, `crates/execution/Cargo.toml`, and `crates/risk/Cargo.toml` still expose Python, PyO3, and Cython surfaces. | RREM/release gates only; RCORE-012 does not authorize deletion. |

RCORE-012 therefore closes the RCORE-010/RCORE-011 inventory loop as a scoped
runtime gate record:

- Safe to claim now: focused Rust assertions for EROL-002 and EROL-003.
- Safe to claim as existing partial evidence only: EROL-004 and EROL-008.
- Not safe to claim as release-ready: EROL-001, EROL-005, EROL-006, EROL-007,
  EROL-009, and EROL-010.

This keeps `removal_allowed = false` and prevents the high-risk runtime gap
inventory from being treated as complete Rust-only release evidence.

## Release Gate Decision

RCORE-010 created the inventory and RCORE-011 adds targeted RiskEngine routing
test evidence.

- `removal_allowed = false`
- No Python/PyO3/Cython/FFI deletion is allowed here.
- No execution, risk, order, matching, emulator, reconciliation, position,
  portfolio, adapter, or public API behavior changes are allowed here.
- RCORE-011 covers risk accept/deny routing assertions for EROL-002 and
  EROL-003.
- RCORE-012 records explicit scope deferrals for the remaining execution,
  matching, emulator, reconciliation, position trace, portfolio/PnL, and
  Python/PyO3/Cython removal gaps. It does not convert those deferrals into
  release-ready evidence.
- This decision does not mark the Rust-only release gate as passed. It records
  the execution/risk/order lifecycle gap map that later high-risk tasks must
  either close or defer with evidence.
