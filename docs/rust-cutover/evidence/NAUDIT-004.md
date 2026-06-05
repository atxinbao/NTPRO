# NAUDIT-004 Evidence - Product-reachable runtime panic cleanup

Date: 2026-06-05
Executor: Codex
Task: NAUDIT-004
Risk: high
Status: REVIEW_REQUIRED

## Goal

清理审计中点名的产品可达 `panic!()` 路径，避免交易/回测运行时因为可预期的不支持输入或缺失关联状态直接崩进程。

## Changed Files

- `.agentflow/leases/NAUDIT-004.json`
- `.agentflow/state/task_status.json`
- `crates/backtest/src/exchange.rs`
- `crates/execution/src/matching_engine/engine.rs`
- `crates/execution/tests/matching_engine.rs`
- `docs/rust-cutover/evidence/NAUDIT-004.md`

## Behavior Changes

- Matching engine:
  - `MARK` price bar no longer panics. It is logged as unsupported and ignored.
  - Missing OTO parent order no longer panics. The child order is rejected with an explicit reason.
  - OTO parent found but not marked `Oto` no longer risks unwrap panic. The child order is rejected with an explicit reason.
  - Missing OCO/OUO linked order during order validation no longer panics. The order is rejected with an explicit reason.
  - Missing linked order during contingent cancellation no longer panics. It logs an error and continues cancellation of the current order.
- Backtest exchange:
  - Unsupported `QueryOrder` / `QueryAccount` commands are logged and ignored instead of entering a panic path.
  - Inflight command generation without a latency model returns `None` and logs an error instead of panicking.
  - Missing matching engine during command processing is logged as an error instead of aborting the process.

## Public API Impact

No public Rust API signature changed.

`SimulatedExchange::send` still returns `()`. Unsupported query commands remain unsupported; this PR only changes the failure mode from process abort to explicit logged no-op.

## Migration Note

No user-facing migration is required. Existing callers should not rely on process aborts for unsupported commands or missing contingent state.

## Validation

Commands were run with the project Rust toolchain from `scripts/ai/toolchain_env.sh`.

Because another workspace-wide cargo command was already using the default target artifact lock, targeted cargo tests were run with `CARGO_TARGET_DIR=target/naudit-004` to avoid corrupting or interrupting that process.

### Targeted smoke

```bash
source scripts/ai/toolchain_env.sh && CARGO_TARGET_DIR=target/naudit-004 cargo test -p nautilus-execution --test matching_engine without_panic
```

Result: passed.

Summary:

- `test_process_mark_bar_ignored_without_panic`
- `test_process_order_rejects_missing_oto_parent_without_panic`
- `test_process_order_rejects_missing_linked_order_without_panic`
- `test_process_cancel_skips_missing_linked_order_without_panic`

```bash
source scripts/ai/toolchain_env.sh && CARGO_TARGET_DIR=target/naudit-004 cargo test -p nautilus-backtest unsupported_command
```

Result: passed.

Summary:

- `test_process_query_account_message_ignores_unsupported_command_without_panic`
- `test_send_query_order_with_latency_model_ignores_unsupported_command`

```bash
source scripts/ai/toolchain_env.sh && CARGO_TARGET_DIR=target/naudit-004 cargo test -p nautilus-backtest without_latency_model_returns_none
```

Result: passed.

Summary:

- `test_generate_inflight_command_without_latency_model_returns_none`

### Required validation

```bash
source scripts/ai/toolchain_env.sh && CARGO_TARGET_DIR=target/naudit-004 cargo test -p nautilus-execution
```

Result: passed.

Summary: `nautilus-execution` unit tests, integration tests, and doc tests passed. Matching engine integration tests reported `169 passed; 0 failed; 3 ignored`.

```bash
source scripts/ai/toolchain_env.sh && CARGO_TARGET_DIR=target/naudit-004 cargo test -p nautilus-backtest
```

Result: passed.

Summary: `nautilus-backtest` unit tests, integration tests, golden trace backtest, and doc tests passed.

```bash
git diff --check
```

Result: passed.

```bash
scripts/ai/verify_fast.sh
```

Result: passed.

Summary:

- `cargo 1.95.0`
- `rustc 1.95.0`
- `cargo fmt --check` passed
- workspace cargo check skipped by fast-smoke default
- clippy skipped by fast-smoke default

## Residual Risk

- This PR does not implement MARK bar execution. It only makes the unsupported path non-crashing.
- This PR does not implement backtest query command semantics. `QueryOrder` and `QueryAccount` are still unsupported in `SimulatedExchange`.
- This PR does not complete broader runtime panic audit outside the NAUDIT-004 scoped paths.
- `cargo test` was run with an isolated target dir because another cargo process held the default target lock. Test behavior is identical, but the artifact path differs.

## Rollback Plan

Revert this PR to restore prior behavior. The main functional risk of rollback is reintroducing process aborts for the scoped MARK bar, contingent order, unsupported command, and missing latency-model paths.
