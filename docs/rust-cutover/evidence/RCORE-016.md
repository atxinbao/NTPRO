# RCORE-016 Evidence - Inventory Rust Trading Strategy / Algorithm API Gaps

Date: 2026-05-30
Executor: Codex
Task ID: RCORE-016
Risk: high

## Summary

Inventoried the Rust trading strategy and execution algorithm API surface for
the Rust-only cutover gate.

The inventory records that Rust already has in-process strategy authoring,
execution algorithm authoring, backtest/trader registration, a bundled TWAP
algorithm, five example Rust strategies, one example Rust data actor, a runnable
Rust EMA-cross backtest smoke, and a Rust plug-in strategy surface.

The remaining gaps are product and parity gates rather than a single local code
bug: Rust CLI strategy selection is not wired, arbitrary config-to-strategy
mapping is not defined, algorithm-specific config remains partially stringly
typed, execution algorithm plug-ins are not implemented, public Rust examples do
not yet cover all strategies, no strategy-plus-algorithm golden trace replay
exists, and Python/PyO3/Cython strategy surfaces remain gated for later removal.

## Files Changed

- `docs/rust-cutover/inventory/trading_strategy_algorithm_api.md`
- `docs/rust-cutover/inventory/README.md`
- `docs/rust-cutover/evidence/RCORE-016.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-016.json`

## Commands Run

Task setup and scope:

```bash
python3 scripts/control/dispatch_next.py --dry-run --max-risk high
python3 scripts/control/dispatch_next.py --max-risk high
git switch -c ai/RCORE-016-inventory-rust-trading-strategy-and-algorithm-api-gaps
python3 scripts/ai/lease.py claim RCORE-016 --branch ai/RCORE-016-inventory-rust-trading-strategy-and-algorithm-api-gaps --agent-id Codex --path docs/rust-cutover/tasks/RCORE-016.md --path docs/rust-cutover/evidence/RCORE-016.md --path docs/rust-cutover/inventory/ --path .agentflow/state/task_status.json --path .agentflow/leases/RCORE-016.json
```

Context and inventory:

```bash
rg --files crates/trading crates | rg '(^crates/trading/|strategy|strategies|algorithm|algo|actor)'
find examples -maxdepth 4 -type f | sort | rg -i 'strategy|strategies|algorithm|algo|trading|backtest|live'
find crates/backtest/examples crates/live/examples crates/trading/examples examples/rust -maxdepth 3 -type f
find crates/trading/src/examples/strategies -mindepth 1 -maxdepth 1 -type d
find crates/trading/src/examples/actors -mindepth 1 -maxdepth 1 -type d
rg -n "strategy|algorithm|exec_algorithm|Twap|EmaCross|nautilus_strategy|plugin" docs/rust-cutover docs/getting_started examples/rust crates/trading/README.md crates/plugin/README.md
rg -n "class .*TWAP|TWAPExecAlgorithm|horizon_secs|interval_secs|exec_algorithm_params|exec_algorithm_id" nautilus_trader examples/backtest docs/getting_started crates/trading/src
sed -n '1,260p' crates/trading/src/lib.rs
sed -n '1,260p' crates/trading/src/strategy/core.rs
sed -n '1,220p' crates/trading/src/strategy/config.rs
sed -n '1,180p' crates/trading/src/strategy/mod.rs
sed -n '1,280p' crates/trading/src/algorithm/core.rs
sed -n '1,220p' crates/trading/src/algorithm/config.rs
sed -n '1,340p' crates/trading/src/algorithm/twap.rs
sed -n '1,260p' crates/backtest/examples/engine_ema_cross.rs
sed -n '1,220p' crates/backtest/examples/node_ema_cross.rs
sed -n '1,180p' crates/live/examples/sandbox_node_smoke.rs
sed -n '1,260p' docs/rust-cutover/product/RUST_API_ENTRYPOINTS.md
sed -n '145,162p' docs/rust-cutover/product/BACKTEST_CLI_CONTRACT.md
sed -n '35,48p' crates/plugin/README.md
```

Required validation:

```bash
scripts/ai/verify_fast.sh
cargo test -p nautilus-trading --features examples
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-trading --features examples
```

Final local checks:

```bash
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RCORE-016.json
python3 -m json.tool /Users/mac/.codex/shrimp-data/NTPRO/tasks.json
git diff --check
```

## Command Results

- `dispatch_next.py --dry-run --max-risk high`: identified RCORE-016 as the next
  eligible high-risk task and branch
  `ai/RCORE-016-inventory-rust-trading-strategy-and-algorithm-api-gaps`.
- `dispatch_next.py --max-risk high`: failed before local mutation because
  `git fetch --prune origin` returned `Empty reply from server`. The branch,
  lease, agentflow state, and isolated Shrimp queue were then updated manually
  following the same dispatch contract.
- Code-index MCP was requested through tool discovery but was not exposed as a
  callable tool in this session; repository inspection used local `rg`, `find`,
  and direct file reads instead.
- `scripts/ai/verify_fast.sh`: passed with `== verify_fast complete ==`. The
  script ran toolchain lookup and `cargo fmt --check`; cargo check and clippy
  remained skipped by the script defaults.
- First `cargo test -p nautilus-trading --features examples`: failed before
  compiling tests because default `rustc 1.87.0` does not meet the workspace
  MSRV (`nautilus-*` crates require `rustc 1.95.0`).
- Retried with the local 1.95.0 Rust toolchain:
  `cargo test -p nautilus-trading --features examples` passed; 315 library
  tests passed, 0 failed, and 4 doctests were ignored.
- `validate_agentflow_roles.py`: passed.
- JSON validation for `.agentflow/state/task_status.json`,
  `.agentflow/leases/RCORE-016.json`, and the isolated NTPRO Shrimp queue:
  passed.
- `git diff --check`: passed.
- The inventory scan found:
  - `Strategy`, `StrategyCore`, `StrategyConfig`, and
    `ImportableStrategyConfig` exported by `nautilus-trading`;
  - `ExecutionAlgorithm`, `ExecutionAlgorithmCore`, `ExecutionAlgorithmConfig`,
    `ImportableExecAlgorithmConfig`, and `TwapAlgorithm` exported by
    `nautilus-trading`;
  - five Rust example strategies under `crates/trading/src/examples/strategies`;
  - one Rust example data actor under `crates/trading/src/examples/actors`;
  - runnable Rust backtest examples under `crates/backtest/examples`;
  - a runnable Rust sandbox node smoke under `crates/live/examples`;
  - Python/PyO3/Cython comparison surfaces still present under
    `crates/trading/src/python/**`, `nautilus_trader/trading/**`,
    `nautilus_trader/examples/**`, and Python examples/docs.

## Tests Added Or Updated

No Rust tests were added. RCORE-016 is inventory-only.

The test gap is explicitly mapped to RCORE-017, which must run or identify the
Rust tests covering strategy and algorithm API surfaces.

## Behavior Impact

No runtime behavior changed. No strategy logic, algorithm logic, execution
routing, risk behavior, portfolio behavior, adapter behavior, persistence
behavior, public API shape, Python API, PyO3 binding, or Cython surface changed.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR only adds inventory and evidence.

## Gate Status

RCORE-016 is high risk because strategy and execution algorithm APIs sit on the
trading-semantic path: strategy selection, order submission, algorithm routing,
position lifecycle, and golden trace parity all depend on this boundary.

This PR must stop at `REVIEW_REQUIRED`. Auto-merge must not be enabled.

## Rollback Plan

Revert this inventory file, the inventory README entry, this evidence file, and
the RCORE-016 task state/lease updates. No runtime, persisted data, adapter,
schema, Python, PyO3, Cython, or public API rollback is required.
