# RBTL-001 Evidence - Inventory Rust backtest runtime gaps

- Date: 2026-05-30
- Executor: Codex
- Task ID: RBTL-001
- Owner role: rust_core_runtime_agent
- Review role: verification_release_gatekeeper
- Risk level: high
- Branch: ai/RBTL-001-inventory-rust-backtest-runtime-gaps
- Gate status: REVIEW_REQUIRED; auto-merge is not enabled for this task.

## Goal

Inventory `crates/backtest` for Rust-only data loading, config, execution, and result gaps. Confirm which Rust backtest paths are already usable and scope blockers with evidence before later runtime or CLI work.

## Files changed

- `docs/rust-cutover/evidence/RBTL-001.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RBTL-001.json`

No runtime, CLI, adapter, Python, PyO3, Cython, or Cargo files were changed.

## Plain summary

Rust backtesting is not empty: direct `BacktestEngine` execution works, `BacktestNode` can load synthetic catalog data, and `BacktestResult` exposes run statistics. The product gap is above that layer: the Rust CLI still has only `backtest validate/run` stubs, backtest config types are not yet serde/TOML-ready, and strategies are still wired manually in Rust examples instead of through a stable user config.

## Current usable Rust paths

- Direct engine path: `BacktestEngine::new`, `add_venue`, `add_instrument`, `add_data`, `add_strategy`, `run`, `end`, `clear_data`, and `get_result` are available from Rust.
- Catalog/node path: `BacktestNode` is available behind the `streaming` feature and can load Parquet catalog data with `BacktestDataConfig`.
- Data coverage in node tests includes quotes, trades, bars, order book deltas, order book depth, price updates, instrument status, and instrument close inputs.
- Result surface exists through `BacktestResult`, including run config ID, instance ID, total events, total orders, total positions, stats, and elapsed time.
- Existing Rust examples cover direct synthetic engine smoke and synthetic Parquet catalog smoke.

## Scoped blockers

| ID | Gap | Evidence | Next likely owner |
| --- | --- | --- | --- |
| BTL-001 | Rust backtest config is not yet a stable file contract. `BacktestEngineConfig`, `BacktestVenueConfig`, `BacktestDataConfig`, and `BacktestRunConfig` are Rust builders, but do not derive serde `Serialize`/`Deserialize`. | `crates/backtest/src/config.rs` | RBTL-002 / NCLI follow-up |
| BTL-002 | Strategy, actor, and execution algorithm wiring is still manual for Rust examples. `BacktestRunConfig` does not carry strategy specs; examples call `get_engine_mut(...).add_strategy(...)` after node build. | `crates/backtest/examples/node_ema_cross.rs` | RBTL-002 / TSAA follow-up |
| BTL-003 | CLI product path is still a blocker. `nautilus backtest validate` and `nautilus backtest run` are exposed but intentionally return "not implemented yet". | `crates/cli/src/lib.rs`, `crates/cli/src/opt.rs` | NCLI/RPROD follow-up |
| BTL-004 | Example config and run artifact contract are missing. `examples/rust/backtest/ema_cross.toml` is referenced by CLI checks but does not exist yet. | CLI validation commands below | NCLI/RPROD follow-up |
| BTL-005 | `BacktestNode::new` currently supports only one run config because kernel `MessageBus` is thread-local singleton based. | `crates/backtest/src/node.rs` | RBTL follow-up |
| BTL-006 | Python/PyO3 remains present behind the `python` feature and is not removed by this task. | `crates/backtest/src/lib.rs`, `crates/backtest/Cargo.toml` | gated NREM/CY work only |
| BTL-007 | Default parallel full verification can hit an existing global logger race. The exact failed test passes alone, and full verification passes with `RUST_TEST_THREADS=1`. | command log below | Verification follow-up |

## Commands run

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo run -p nautilus-backtest --features examples --example engine-ema-cross
```

Result: passed. Summary output included `Data elements: 745`, `Iterations: 745`, `Total events: 36`, `Total orders: 12`, and `Total positions: 12`.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo run -p nautilus-backtest --features examples,streaming --example node-ema-cross
```

Result: passed. Summary output included catalog write of `745` quote ticks, chunked replay, `Iterations: 745`, `Total events: 36`, `Total orders: 12`, and `Total positions: 12`.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-backtest --features streaming --test backtest_node
```

Result: passed. `40 passed; 0 failed`.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo run -q -p nautilus-cli -- backtest --help
```

Result: passed. Help lists `validate`, `run`, and `help`.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo run -q -p nautilus-cli -- backtest validate --config examples/rust/backtest/ema_cross.toml
```

Result: expected blocker, exit 1. Message: `backtest validate is defined but not implemented yet`.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo run -q -p nautilus-cli -- backtest run --config examples/rust/backtest/ema_cross.toml --run-id ema-cross --output runs/ema-cross
```

Result: expected blocker, exit 1. Message: `backtest run is defined but not implemented yet`.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc scripts/ai/verify_full.sh
```

Result: failed in `engine::tests::test_add_strategy_registers_strategy_with_trader` because Nautilus logging attempted to initialize after another logger was already registered.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-backtest --lib engine::tests::test_add_strategy_registers_strategy_with_trader -- --exact
```

Result: passed. `1 passed; 0 failed`.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc RUST_TEST_THREADS=1 scripts/ai/verify_full.sh
```

Result: passed. Output ended with `== verify_full complete ==`.

```bash
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RBTL-001.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc scripts/ai/verify_fast.sh
```

Result: passed. `verify_fast` completed with cargo check and clippy skipped by fast-mode defaults.

## Behavior impact

No behavior changed. This PR only records the inventory, blocker scope, validation evidence, and agentflow task state.

## Public API impact

No public API changed.

## Migration note status

No migration note is required for this inventory-only PR.

## Rollback plan

Revert this PR to remove the RBTL-001 evidence file and restore the previous agentflow task state. No runtime rollback is required because no runtime code was changed.

## Review status

High-risk task. This must stop at `REVIEW_REQUIRED` and wait for Verification & Release Gatekeeper review before merge. Auto-merge must remain disabled.
