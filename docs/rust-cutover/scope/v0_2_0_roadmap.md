# NTPRO v0.2.0 Roadmap

Date: 2026-06-04
Executor: Codex
Status: Superseded planning record only

## Superseded By V02 Scope Decision

This document is retained as historical planning context. It is not the active
execution source for the current v0.2 queue.

The active v0.2 scope is:

```text
Local Multi-Node Runtime Foundation
```

Use `docs/rust-cutover/scope/v0_2_local_multi_node_runtime.md` and `V02-001`
through `V02-010` for executable task scope.

The product-hardening items below were useful during post-release cleanup and
design readiness work, but they must not be used to start Dashboard UI, control
API endpoint implementation, production exchange connectivity, manual order
entry, distributed deployment, or release/tag work in the V02 queue.

## Purpose

NTPRO v0.1.0 is the first formal Rust-only release point. The next phase should
not mix more removal work into product development, and should not start the
Operator Dashboard MVP as the primary v0.2.0 goal.

The v0.2.0 target is:

```text
Rust-only product hardening and operator-ready foundation
```

This means v0.2.0 should stabilize the public release surface, Rust CLI user
path, installation path, adapter support matrix, verification path,
architecture boundaries, and future observability/control contracts.

Operator Dashboard MVP is deferred to v0.3.0 or a later v0.2.x follow-up after
the product foundation is stable.

## v0.2.0 Scope

In scope:

- public release surface cleanup;
- post-release gap inventory;
- toolchain and verification path hardening;
- Rust CLI help contract;
- minimal Rust backtest, sandbox, and live-init user paths;
- adapter support matrix;
- release binary and install path decision;
- trace and performance expansion plan;
- architecture map and module contracts;
- node lifecycle, observability, and control API contracts;
- Dashboard MVP scope contract only.

Out of scope:

- Dashboard UI implementation;
- runtime telemetry implementation;
- node control API implementation;
- live control wiring;
- manual order entry;
- strategy parameter hot reload;
- multi-user permissions;
- Docker delivery as a v0.2.0 requirement;
- new Python/PyO3/Cython product surfaces;
- additional removal work unless a specific cleanup task is approved.

## Task Plan

### RHARD-000: Public release surface cleanup

Goal: clean the public appearance after the formal Rust-only release.

Work:

- Update README wording from release-candidate language to formal Rust-only
  release workspace language.
- Clean remaining Python/PyPI user-path references from docs.
- State that Python is allowed only for local repository helper scripts, not as
  a product entrypoint.
- Keep release notes, installation docs, and getting-started docs consistent.

Deliverables:

- README update.
- Installation docs update.
- Public surface audit evidence.

Risk: low.

### RHARD-001: Post-release gap list

Goal: list remaining gaps after v0.1.0.

Work:

- Inventory CLI gaps.
- Inventory examples gaps.
- Inventory docs gaps.
- Inventory adapter gaps.
- Inventory verification gaps.
- Inventory architecture gaps.

Deliverable:

- `docs/rust-cutover/post-release-gap-list.md`.

Risk: low.

### RHARD-002: Toolchain and verification path hardening

Goal: prevent local verification from using the wrong Rust compiler.

Work:

- Document the required Rust 1.95.0 toolchain path.
- Explain `rustup override set 1.95.0`.
- Avoid accidental use of Homebrew `rustc` when running release checks.
- Add an optional preflight check if needed.

Deliverables:

- Toolchain documentation.
- Verification documentation update.
- Optional preflight check.

Risk: low to medium.

### RHARD-007: Verification cleanup

Goal: make verification choices clear for users and agents.

Work:

- Document when to run `verify_fast`.
- Document when to run `verify_full`.
- Document when to run `verify_release`.
- Document `check_rust_only_runtime`.
- Document `check_cython_removed`.
- Document golden trace checks.
- Explain why release build is slow and expected to take substantially longer.

Deliverable:

- Verification guide update.

Risk: low.

### RHARD-003: CLI help contract

Goal: stabilize the Rust CLI product entrypoints.

Work:

- Validate and document help contracts for:
  - `backtest`;
  - `sandbox`;
  - `live`;
  - `data`;
  - `config`;
  - `database`.

Deliverables:

- CLI contract documentation.
- CLI help evidence.

Risk: medium.

### RHARD-006: Backtest CLI minimal path

Goal: let a user run one minimal backtest path.

Work:

- Define input data.
- Define config.
- Define command.
- Define output path.
- Define expected result.

Deliverables:

- Backtest CLI docs.
- Backtest example.
- Smoke evidence.

Risk: medium.

### RHARD-004: Sandbox demo

Goal: provide a minimal sandbox or paper-like run path.

Work:

- Use simulated data.
- Use simulated execution.
- Show node start and stop.
- Show event flow.
- Expose basic risk, portfolio, and cache state.

Deliverables:

- Sandbox example.
- Quickstart documentation.
- Smoke evidence.

Risk: medium.

### RHARD-005: Live init smoke

Goal: verify live node initialization and shutdown without real orders.

Work:

- Define live config.
- Initialize kernel.
- Register adapter.
- Start and shut down.

Deliverables:

- Live init smoke example.
- Documentation.

Risk: medium.

### NADAPT-001: Adapter support matrix

Goal: make adapter support status explicit.

Categories:

- supported;
- sandbox-only;
- fixture-only;
- deferred;
- removed.

Adapters to classify:

- Binance;
- OKX;
- Bybit;
- Coinbase;
- Databento;
- Deribit;
- dYdX;
- Hyperliquid;
- Interactive Brokers;
- Kraken;
- Polymarket;
- Tardis;
- Sandbox;
- other workspace adapters.

Deliverable:

- `docs/integrations/adapter_support_matrix.md`.

Risk: medium.

### NBIN-001: Release binary and install path

Goal: define how users install and run the Rust CLI.

Work:

- Decide source build path.
- Decide whether `cargo install` is supported.
- Decide release artifact strategy.
- Decide binary naming.
- Decide platform scope.
- Explicitly defer Docker delivery for now.

Deliverables:

- Install and run documentation.
- Binary and release artifact decision record.

Risk: medium.

### NTRACE-001: Trace and performance expansion plan

Goal: define v0.2.0 trace and performance evidence expansion.

Work:

- Backtest trace expansion.
- Live and sandbox lifecycle trace expansion.
- Data source trace expansion.
- Execution order lifecycle trace expansion.
- Risk rejection trace expansion.
- Adapter payload trace expansion.
- Performance smoke scope.

Deliverable:

- `docs/rust-cutover/trace_performance_expansion_plan.md`.

Risk: low to medium.

## Architecture Foundation

### NARCH-001: Rust-only architecture map

Goal: document the current Rust-only architecture.

Areas:

- Product Surface;
- Node Runtime;
- System Kernel / Trader;
- DataEngine;
- ExecutionEngine;
- RiskEngine;
- Portfolio;
- MessageBus;
- Cache;
- Persistence / Event Store;
- Adapter Layer;
- Verification Gates.

Risk: low.

### NARCH-006: Module boundary audit

Goal: audit current module boundaries before refactoring.

Work:

- Check whether current crates match the target architecture boundaries.
- Identify mixed concerns.
- Identify internal implementation details that Dashboard code must not read
  directly.
- Identify state that should later be exposed through a stable telemetry
  surface.

Risk: low.

### NARCH-002: Module contracts

Goal: write contracts for core modules.

Each contract should define:

- responsibilities;
- inputs;
- outputs;
- state;
- lifecycle;
- error model;
- dependency boundaries;
- candidate dashboard-observable fields.

Risk: medium.

### NARCH-003: Node lifecycle state machine

Goal: define the node lifecycle model.

States:

```text
stopped
starting
running
pausing
paused
resuming
stopping
error
```

Risk: medium.

### NARCH-004: Observability state model

Goal: define the future dashboard-readable state model.

Areas:

- system status;
- data source status;
- execution gateway status;
- risk status;
- portfolio summary;
- alert summary.

This task defines the model only. It does not implement UI.

Risk: medium.

### NARCH-005: Control API contract

Goal: define control actions without implementing live control.

Actions:

```text
start
stop
restart
pause_trading
resume_trading
reconnect_data
reconnect_execution
```

This task is contract-only. Runtime implementation belongs to a later dashboard
or control-plane phase.

Risk: medium to high.

## Dashboard Boundary

### NDASH-001: Dashboard MVP scope contract

Goal: lock the first Dashboard MVP scope before implementation starts.

The first version may include:

- status viewing;
- alert viewing;
- node lifecycle viewing;
- start, stop, pause, and resume controls.

The first version must not include:

- manual order entry;
- strategy parameter hot reload;
- multi-user permissions;
- complex asset management;
- full trading frontend scope;
- Docker delivery as a requirement.

Risk: low.

## Execution Order

```text
1. RHARD-000
2. RHARD-001
3. RHARD-002
4. RHARD-007
5. RHARD-003
6. RHARD-006
7. RHARD-004
8. RHARD-005
9. NADAPT-001
10. NBIN-001
11. NTRACE-001
12. NARCH-001
13. NARCH-006
14. NARCH-002
15. NARCH-003
16. NARCH-004
17. NARCH-005
18. NDASH-001
```

## Deferred From v0.2.0 Mainline

The following tasks should not enter the v0.2.0 mainline:

```text
NOBS-001
NOBS-002
NOBS-003
NOBS-004
NCTRL-001
NDASH-002
NDASH-003
NDASH-004
NDASH-005
```

Reason: these tasks start runtime telemetry implementation, control API
implementation, Dashboard UI implementation, or node lifecycle control. That
would pull v0.2.0 away from product hardening into Dashboard development.

## Planning Boundary

This document is a roadmap record. It does not create Shrimp tasks, leases,
branches for individual tasks, or implementation approval.

Before execution starts, approved tasks should be entered into the isolated
NTPRO Shrimp queue and should follow `docs/rust-cutover/TASK_EXECUTION.md`.
