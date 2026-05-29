# Rust Trading Strategy / Algorithm API Gap Inventory

Date: 2026-05-30
Executor: Codex
Task ID: RCORE-016

## Scope

This inventory covers Rust strategy authoring, strategy registration, execution
algorithm authoring, strategy examples, execution algorithm examples, and the
Rust-only product path needed to run strategy logic without Python, PyO3, or
Cython as product surfaces.

The task is inventory-only. It does not change runtime behavior, trading
semantics, public APIs, adapters, persistence formats, Python/PyO3/Cython
surfaces, or Cargo feature behavior.

## Rust Surfaces Inspected

| Area | Files | Current Rust surface |
| --- | --- | --- |
| Trading crate exports | `crates/trading/src/lib.rs` | `nautilus-trading` exports `Strategy`, `StrategyCore`, `StrategyConfig`, `ImportableStrategyConfig`, `ExecutionAlgorithm`, `ExecutionAlgorithmCore`, `ExecutionAlgorithmConfig`, `ImportableExecAlgorithmConfig`, `TwapAlgorithm`, and `TwapAlgorithmConfig`. Python bindings are behind the `python` feature; default features are empty. |
| Strategy runtime API | `crates/trading/src/strategy/{mod,core,config}.rs` | Rust strategies are `DataActor`s with order submission, order-list submission, modify/cancel, cancel-all, position close, account/order query, order/position event handling, GTD expiry, and market-exit helpers. `StrategyCore` owns registration state, order factory, order manager, portfolio access, strategy ID, order ID tag, and market-exit state. |
| Execution algorithm runtime API | `crates/trading/src/algorithm/{mod,core,config,twap}.rs` | Rust execution algorithms are `DataActor`s with command dispatch, child-order spawn helpers, modify/cancel helpers, strategy event subscriptions, default order/position callbacks, and one bundled `TwapAlgorithm`. |
| Backtest integration | `crates/backtest/src/engine.rs`, `crates/backtest/examples/{engine_ema_cross,node_ema_cross}.rs` | `BacktestEngine` can add Rust strategies, data actors, and execution algorithms. Existing Cargo examples run Rust `EmaCross` through the engine and node paths. |
| Trader integration | `crates/system/src/trader.rs` | `Trader` registers strategies in component and actor registries, wires order/position event subscriptions, registers market-exit control endpoints, and registers execution algorithm `{id}.execute` endpoints. |
| Rust examples | `crates/trading/src/examples/**`, `examples/rust/**` | The `examples` feature exposes five Rust strategy examples and one data actor example. The public `examples/rust` tree currently documents Rust command/example contracts and points to runnable Cargo smokes rather than full CLI-driven strategy runs. |
| Rust plug-in strategy surface | `crates/plugin/src/surfaces/strategy.rs`, `crates/plugin/src/bridge/strategy.rs`, `crates/plugin/README.md` | A Rust plug-in strategy surface exists and is tested separately. It is explicitly early-alpha, strategy-only, and not a completed execution-algorithm plug-in surface. |
| Python/PyO3/Cython comparison surfaces | `crates/trading/src/python/**`, `nautilus_trader/trading/**`, `nautilus_trader/examples/**`, `docs/getting_started/**`, `examples/backtest/**`, `examples/live/**` | Legacy Python and PyO3 strategy/config/example surfaces remain present and include importable strategy configs, Python strategy factories, Cython strategy runtime, and Python TWAP strategy/algorithm examples. They are comparison or migration surfaces until removal gates approve deletion. |

## Current Rust Evidence

| Behavior | Evidence |
| --- | --- |
| Strategy trait command surface exists | `crates/trading/src/strategy/mod.rs` provides default Rust methods for submit, submit list, modify, cancel, cancel-all, close-position, close-all, query-account, query-order, event dispatch, market exit, and GTD expiry. |
| Execution algorithm trait command surface exists | `crates/trading/src/algorithm/mod.rs` provides `ExecutionAlgorithm::execute`, spawn helpers for market/limit/market-to-limit orders, modify/cancel helpers, event routing, and strategy event subscription helpers. |
| TWAP exists in Rust | `crates/trading/src/algorithm/twap.rs` implements a `TwapAlgorithm` over `ExecutionAlgorithm`; it parses `horizon_secs` and `interval_secs` from order `exec_algorithm_params`. |
| Backtest can register Rust strategies and algorithms | `BacktestEngine::add_strategy` and `BacktestEngine::add_exec_algorithm` forward to `Trader`. |
| Trader wires runtime lifecycle | `Trader::add_strategy` registers strategy core, component/actor registries, order and position subscriptions, and market-exit endpoint; `Trader::add_exec_algorithm` registers an algorithm endpoint for routed trading commands. |
| Runnable Rust strategy smoke exists | `crates/backtest/examples/engine_ema_cross.rs` runs synthetic AUD/USD quotes through Rust `BacktestEngine` and Rust `EmaCross`; RPROD-011 recorded a passing Cargo example smoke. |
| Runnable Rust sandbox smoke exists | `crates/live/examples/sandbox_node_smoke.rs` constructs a Rust sandbox `LiveNode`; RPROD-012 recorded a passing Cargo example smoke. |
| Rust example strategy set exists | `crates/trading/src/examples/strategies/` contains `ema_cross`, `grid_mm`, `composite_market_maker`, `delta_neutral_vol`, and `hurst_vpin_directional`, each with strategy/config modules and tests. |
| Plug-in strategy boundary has test coverage | `crates/plugin/tests/surface_alignment.rs`, `hook_dispatch.rs`, `strategy_execution_dispatch.rs`, `host_vtable_dispatch.rs`, and related tests cover the Rust plug-in strategy surface and host dispatch boundary. |

## Gap Matrix

| Gap | Status | Evidence | Release impact |
| --- | --- | --- | --- |
| TSAA-001 | Open: Rust CLI config cannot select and run arbitrary Rust strategies yet. | `docs/rust-cutover/product/BACKTEST_CLI_CONTRACT.md` states `strategy` is the first CLI run-path blocker and needs a Rust-native strategy registry or explicitly scoped strategy example contract. `examples/rust/backtest/README.md` says `backtest validate` and `backtest run` still return blockers until config parsing, strategy selection, and runtime wiring exist. | Blocks treating `nautilus backtest run` as a Rust-only strategy product path. |
| TSAA-002 | Partial: in-process Rust strategy authoring works, but a stable product strategy registry is not defined. | Rust users can compile strategies in-process and pass concrete types to `BacktestEngine::add_strategy`, but `ImportableStrategyConfig` still models path strings and the CLI product contract has no Rust registry mapping from config to concrete strategy constructors. | Blocks config-driven Rust product workflows and migration from Python importable strategy configs. |
| TSAA-003 | Partial: Rust execution algorithm API exists, but product-facing algorithm selection is still incomplete. | `ExecutionAlgorithm` and `TwapAlgorithm` exist, and `BacktestEngine::add_exec_algorithm` registers concrete algorithms. `TwapAlgorithmConfig` aliases the generic `ExecutionAlgorithmConfig`; TWAP behavior still relies on stringly `exec_algorithm_params` carried by orders for `horizon_secs` and `interval_secs`. | Blocks typed Rust config validation for algorithm-specific parameters and CLI-driven algorithm wiring. |
| TSAA-004 | Open: execution algorithm plug-in surface is not implemented. | `crates/plugin/README.md` marks `Strategy` as shipped and `Execution algorithm` as `Not yet`. | Blocks external Rust plug-ins from providing execution algorithms through the same product boundary as strategies. |
| TSAA-005 | Partial: Rust strategy examples exist, but public examples are not yet full product workflows. | `crates/trading/src/examples/strategies/**` contains five strategy examples with tests, while `examples/rust/**` currently documents command contracts and runnable Cargo smokes. The first runnable Rust backtest smoke covers `EmaCross`; other strategies are not yet exposed as CLI/config examples. | Blocks complete Rust-only quickstart coverage for strategy authoring and selection. |
| TSAA-006 | Open: no strategy-plus-algorithm golden trace replay proves semantic parity. | Existing golden traces include schema fixtures, one single-quote Rust backtest replay with zero orders/positions, order-lifecycle schema fixtures, cache/msgbus, live sandbox lifecycle, and adapter payload replay. No trace currently drives a Rust strategy that submits algorithm-routed orders and compares order, position, and PnL outputs. | Blocks release-class confidence for strategy/algorithm semantic parity. |
| TSAA-007 | Open: Python/PyO3/Cython strategy product surfaces remain present. | `crates/trading/src/python/**`, `nautilus_trader/trading/strategy.pyx`, Python strategy examples, and Python getting-started tutorials remain in tree. RCORE-016 does not authorize deletion. | Blocks Python/PyO3/Cython removal until RREM/release gates approve final product-surface deletion. |
| TSAA-008 | Partial: Rust test coverage exists but is not yet curated into the RCORE-017 gate. | Strategy core, algorithm core, TWAP, trader registration, backtest engine, example strategies, and plug-in strategy surfaces all have Rust tests, but RCORE-016 only inventories them. | RCORE-017 must either run and record the relevant Rust tests or explicitly scope any blockers. |

## Non-Goals Preserved

- No Python, PyO3, Cython, `build.py`, or `pyproject.toml` removal.
- No public API changes.
- No trading-semantic changes.
- No changes to strategy, algorithm, adapter, persistence, portfolio, risk, or
  execution runtime behavior.
- No attempt to replace Python importable strategy configs with a Rust registry
  in this inventory task.

## Follow-Up Mapping

| Follow-up | Owner | Scope |
| --- | --- | --- |
| RCORE-017 | Rust Core Runtime Agent | Run or identify Rust tests covering strategy and algorithm API surfaces, including `nautilus-trading`, `nautilus-system`, `nautilus-backtest`, and relevant plug-in strategy tests. |
| RCORE-018 | Rust Core Runtime Agent | Close implementable Rust strategy/algorithm API gaps or explicitly scope deferrals with evidence. |
| RPROD follow-up | Rust Product Surface Agent | Define or implement Rust CLI strategy selection and config-to-strategy mapping. |
| RTRACE follow-up | Verification & Release Gatekeeper | Add executable strategy-plus-algorithm golden trace replay. |
| RREM follow-up | Rust Core Runtime Agent + Gatekeeper | Remove Python/PyO3/Cython strategy surfaces only after Rust product, runtime, adapter, QA, and release gates approve removal. |
