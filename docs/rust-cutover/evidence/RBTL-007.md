# RBTL-007 Evidence - Close Rust Live Config/Client Gaps

Date: 2026-05-30
Executor: Codex

## 大白话说明

这次处理的是 Rust live 配置和 client 注册之间的一个坑：配置里 `data_clients` / `exec_clients` 虽然能写，但 Rust runtime 现在还没有自动根据这些配置创建 adapter 的注册表。以前这类配置可能被带进来但不会真正创建 client，容易让人误以为已经生效。现在改成明确报错，并提示 Rust 调用方用 `LiveNodeBuilder::add_data_client(...)`、`add_exec_client(...)` 或 `add_simulated_exec_client(...)` 注册 client。同时补了一个纯 Rust 测试，证明不用 Python，也能通过 builder 把 data client 和 execution client 注册到 live engines 里。

## Task

- Task ID: RBTL-007
- Goal: Close Rust live config/client gaps
- Risk: medium
- Branch: `ai/RBTL-007-close-rust-live-config-client-gaps`

## Files Changed

- `crates/live/src/config.rs`
- `crates/live/tests/node.rs`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RBTL-007.json`
- `docs/rust-cutover/evidence/RBTL-007.md`

## Summary

Closed the silent-ignore gap for Rust live client configuration maps and added Rust-only coverage for explicit live client factory registration.

## Implementation Notes

- `LiveNodeConfig::validate_runtime_support()` now rejects non-empty `data_clients` and `exec_clients` maps for the Rust runtime.
- The error messages point callers to the supported Rust builder APIs for explicit client registration.
- Added config validation tests for the rejected data and execution client maps.
- Added `serial_tests::test_builder_registers_rust_data_and_exec_client_factories` to prove explicit Rust factories register data and execution clients into the live data/execution engines.
- The test uses local mock clients and factories only; it does not connect to a real exchange and does not require Python, PyO3, or Cython.

## Commands Run

```bash
python3 scripts/control/dispatch_next.py
```

Result: passed. Dispatched RBTL-007 to branch `ai/RBTL-007-close-rust-live-config-client-gaps`.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH \
  RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  cargo fmt --check
```

Result: passed.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH \
  RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  cargo test -p nautilus-live test_live_node_config_rejects -- --nocapture
```

Result: passed. `2 passed; 0 failed`.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH \
  RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  cargo test -p nautilus-live --test node \
  serial_tests::test_builder_registers_rust_data_and_exec_client_factories -- --exact --nocapture
```

Result: passed. The Rust builder registered `TEST-DATA` into the live data engine and `TEST-EXEC` into the live execution engine.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH \
  RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  cargo test -p nautilus-live --test node -- --test-threads=1
```

Result: passed. `28 passed; 0 failed; 0 ignored`.

```bash
scripts/ai/verify_full.sh
```

Result: first attempt failed at clippy with `clippy::unnecessary_literal_bound` in the local mock factory methods added to `crates/live/tests/node.rs`. Fixed by making the mock factory metadata methods return `&'static str`.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH \
  RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  cargo fmt --check

git diff --check

env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH \
  RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  cargo test -p nautilus-live --test node \
  serial_tests::test_builder_registers_rust_data_and_exec_client_factories -- --exact --nocapture
```

Result: passed after the clippy fix.

```bash
scripts/ai/verify_full.sh
```

Result: passed. The run completed with `== verify_full complete ==`.

```bash
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RBTL-007.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

Result: passed. Agentflow JSON and role protocol validation are valid.

## Tests Added or Updated

- Added `config::tests::test_live_node_config_rejects_data_client_maps_for_rust_runtime`.
- Added `config::tests::test_live_node_config_rejects_exec_client_maps_for_rust_runtime`.
- Added `serial_tests::test_builder_registers_rust_data_and_exec_client_factories`.

## Behavior Impact

Rust live runtime now fails fast when `LiveNodeConfig.data_clients` or `LiveNodeConfig.exec_clients` are provided, instead of leaving those maps unused. Existing explicit builder registration remains supported.

## Public API Impact

No public Rust API signatures changed. Runtime validation behavior is stricter for unsupported Rust live client config maps.

## Migration Note

No migration file is required for this internal cutover task. The PR body documents that Rust callers should use the explicit `LiveNodeBuilder` client registration APIs until a native Rust adapter registry exists.

## Rollback Plan

Revert this PR. That restores the previous validation behavior and removes the new Rust client-registration tests and evidence.
