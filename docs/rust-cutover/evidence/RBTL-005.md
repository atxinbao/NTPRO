# RBTL-005 Evidence - Inventory Rust Live Node Gaps

Date: 2026-05-30
Executor: Codex

## 大白话说明

这次没有改 live runtime，只是把 Rust live node 现在能做什么、还缺什么盘清楚。结论是：Rust 这边已经能直接构建 `LiveNode`，能注册 data/execution/sandbox client，能按 data 先、execution 后的顺序启动，也有 handle stop、`ShutdownSystem`、shutdown drain 和 sandbox golden trace 覆盖。缺口也很明确：一批配置现在会被 Rust runtime 主动拒绝；`data_clients` / `exec_clients` 配置能反序列化，但还没有靠配置自动创建 adapter factory 的 Rust 注册表；启动早期连接 data client 时，代码里还标着 TODO，遇到卡住的 connect future 时还不能及时响应 stop 或 shutdown 信号。没有碰 Python、PyO3、Cython，也没有改交易行为。

## Task

- Task ID: RBTL-005
- Goal: Inventory Rust live node gaps
- Risk: medium
- Branch: `ai/RBTL-005-inventory-rust-live-node-gaps`

## Files Changed

- `.agentflow/state/task_status.json`
- `.agentflow/leases/RBTL-004.json`
- `.agentflow/leases/RBTL-005.json`
- `docs/rust-cutover/evidence/RBTL-005.md`

## Summary

Inspected the Rust `nautilus-live` node surface for Rust-only configuration, lifecycle, client registration, and shutdown behavior. This is an inventory-only task; no runtime implementation files were modified.

## Inventory Findings

### Usable Rust live node surface

- `LiveNodeConfig` is a Rust-native serde config model with `deny_unknown_fields`, defaults, kernel config conversion, and runtime-support validation.
- `LiveNodeBuilder` supports `Sandbox` and `Live` environments and rejects `Backtest`.
- `LiveNodeBuilder` provides Rust APIs for registering data clients, execution clients, simulated execution clients, event-store factories, logging, cache, portfolio, data/risk/exec engine configs, and lifecycle timeouts.
- `LiveNode::run()` owns the integrated single-threaded event loop and starts data clients before execution clients, then runs reconciliation and starts the trader.
- `LiveNode::start()` supports a manual lifecycle path for callers that need startup without consuming the runner event loop.
- `LiveNodeHandle` supports cross-context stop requests and state inspection.
- `AsyncRunner` binds time/data/execution channel senders into thread-local storage before clients or event loops need them.
- Sandbox live lifecycle is covered by `crates/live/tests/golden_trace_live_sandbox.rs`, and the Rust example `crates/live/examples/sandbox_node_smoke.rs` proves a no-Python sandbox node construction path.

### Scoped Rust live node gaps

- `LiveNodeConfig.msgbus`, `LiveNodeConfig.streaming`, `LiveNodeConfig.emulator`, and `LiveNodeConfig.loop_debug` are rejected by `validate_runtime_support`; they are not Rust-live-supported yet.
- `LiveDataEngineConfig.qsize`, `LiveRiskEngineConfig.qsize`, and `LiveExecEngineConfig.qsize` are rejected unless left at the default.
- `LiveDataEngineConfig.graceful_shutdown_on_error`, `LiveRiskEngineConfig.graceful_shutdown_on_error`, and `LiveExecEngineConfig.graceful_shutdown_on_error` are rejected because the Rust live runtime does not support those queue-error shutdown semantics yet.
- `LiveExecEngineConfig.snapshot_orders`, `snapshot_positions`, and `purge_from_database` are rejected because the live kernel does not yet wire the required backing database/cache adapter path.
- `LiveNodeConfig.data_clients` and `LiveNodeConfig.exec_clients` are serde/Python-facing config maps, but the Rust builder path still requires explicit factory registration through `add_data_client`, `add_exec_client`, or `add_simulated_exec_client`. There is no Rust adapter factory registry that turns those config maps into clients automatically.
- `LiveNode::build()` rejects `event_store` config without a factory; Rust callers must use `LiveNodeBuilder::with_event_store(...)`.
- `LiveNode::run()` still has a startup TODO before data-client connection: stop handle, shutdown flag, and Ctrl-C monitoring are not wired while `connect_data_clients()` is in-flight, so a hanging connect future can delay shutdown during that phase.

## Suggested Follow-up Tasks

- Add a Rust live adapter factory registry or explicit per-adapter builder helpers so `LiveNodeConfig.data_clients` and `LiveNodeConfig.exec_clients` can be used without Python-side construction.
- Add startup cancellation coverage around `connect_data_clients()` and `connect_exec_clients()` so `LiveNode::run()` can abort promptly on stop/shutdown while a client connect future is pending.
- Decide whether unsupported runtime config fields should remain rejected, be wired natively, or be removed from the Rust-first product surface.
- Add a Rust live node config-to-builder smoke that includes at least one sandbox simulated execution client and documents the supported config subset.

## Commands Run

```bash
git status --short --branch
```

Result: passed. Confirmed branch `ai/RBTL-005-inventory-rust-live-node-gaps` and only task metadata from dispatch was dirty before evidence work.

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RBTL-005.md
sed -n '1,220p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,180p' docs/rust-cutover/AGENT_ROLES.md
```

Result: passed. Confirmed task scope, required `scripts/ai/verify_full.sh`, owner role, review role, and path rules.

```bash
find crates/live -maxdepth 3 -type f | sort
rg -n "struct LiveNodeConfig|validate_runtime_support|add_data_client|add_exec_client|shutdown|connect_data_clients|connect_exec_clients|TODO" crates/live
sed -n '520,860p' crates/live/src/config.rs
sed -n '620,1465p' crates/live/src/node.rs
sed -n '220,520p' crates/live/src/builder.rs
sed -n '1,560p' crates/live/tests/node.rs
sed -n '1,180p' crates/live/examples/sandbox_node_smoke.rs
sed -n '1,180p' crates/live/tests/golden_trace_live_sandbox.rs
```

Result: passed. These reads produced the inventory above.

```bash
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RBTL-004.json >/dev/null
python3 -m json.tool .agentflow/leases/RBTL-005.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

Result: passed. Task state, lease JSON, role protocol, and whitespace checks are valid.

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH \
  RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  RUST_TEST_THREADS=1 scripts/ai/verify_full.sh
```

Result: passed. The full local gate completed successfully, including `verify_fast`, workspace clippy/tests, Rust docs, `nautilus-live` log-global tests, and golden trace validation. Final line: `== verify_full complete ==`.

## Tests Added or Updated

None. This task is inventory/evidence only and does not change runtime code.

## Behavior Impact

None. No live node, adapter, execution, risk, data, or trading behavior changed.

## Public API Impact

None. No public API signatures, Python APIs, PyO3 bindings, or CLI contracts changed.

## Migration Note

Not required. No public API or user-facing config behavior changed.

## Rollback Plan

Revert this PR. That removes the RBTL-005 inventory evidence and task metadata updates only.
