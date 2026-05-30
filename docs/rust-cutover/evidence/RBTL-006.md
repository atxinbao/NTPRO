# RBTL-006 Evidence - Add Rust LiveNode Lifecycle Smoke

Date: 2026-05-30
Executor: Codex

## 大白话说明

这次补了一个很直接的 Rust live node 冒烟测试：用 Rust builder 创建 Sandbox `LiveNode`，不经过 Python，然后调用 `start()`，确认节点进入 `Running`，再调用 `stop()`，确认节点进入 `Stopped`。这个测试不接真实交易所、不发真实订单、不改交易逻辑，只证明 Rust 入口自己可以完成最基本的启动和停止闭环。

## Task

- Task ID: RBTL-006
- Goal: Add Rust LiveNode lifecycle smoke
- Risk: medium
- Branch: `ai/RBTL-006-add-rust-livenode-lifecycle-smoke`

## Files Changed

- `crates/live/tests/node.rs`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RBTL-006.json`
- `docs/rust-cutover/evidence/RBTL-006.md`

## Summary

Added a Rust-only `LiveNode` lifecycle smoke in the existing live node integration test suite. The smoke builds a Sandbox node through `LiveNode::builder`, disables startup reconciliation, starts the node with `start()`, verifies `Running`, stops the node with `stop()`, and verifies `Stopped`.

## Implementation Notes

- The new test is `serial_tests::test_rust_builder_start_stop_smoke_without_python`.
- It lives beside the existing `LiveNode` lifecycle tests in `crates/live/tests/node.rs`.
- It uses `Environment::Sandbox` to prove the Rust builder preserves the sandbox product path.
- It uses `tokio::time::timeout(Duration::from_secs(5), ...)` around both `start()` and `stop()` so lifecycle regressions fail quickly.
- It does not add Python, PyO3, Cython, adapter, CLI, or public API dependencies.

## Commands Run

```bash
python3 scripts/control/dispatch_next.py
```

Result: passed. Dispatched RBTL-006 to branch `ai/RBTL-006-add-rust-livenode-lifecycle-smoke`.

```bash
cargo fmt --check
```

Result: passed.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH \
  RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  cargo test -p nautilus-live --test node \
  serial_tests::test_rust_builder_start_stop_smoke_without_python -- --exact --nocapture
```

Result: passed. The new smoke built a Rust Sandbox `LiveNode`, started it, verified `Running`, stopped it, and verified `Stopped`.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH \
  RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  cargo test -p nautilus-live --test node -- --test-threads=1
```

Result: passed. `27 passed; 0 failed; 0 ignored`.

```bash
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RBTL-006.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

Result: passed. Agentflow JSON files are valid, role protocol validation passed, and the diff has no whitespace errors.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH \
  RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  RUST_TEST_THREADS=1 scripts/ai/verify_full.sh
```

Result: passed. Full verification completed with `== verify_full complete ==`.

## Tests Added or Updated

- Added `serial_tests::test_rust_builder_start_stop_smoke_without_python` in `crates/live/tests/node.rs`.

## Behavior Impact

No production behavior changed. This is test coverage only.

## Public API Impact

None. No public API signatures, Python APIs, PyO3 bindings, CLI commands, or config contracts changed.

## Migration Note

Not required. No public API or user-facing behavior changed.

## Rollback Plan

Revert this PR. That removes the lifecycle smoke test and RBTL-006 task evidence/state updates.
