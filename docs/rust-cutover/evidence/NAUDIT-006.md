# NAUDIT-006 Evidence - Live adapter cancellation contract and mock evidence

Date: 2026-06-05
Executor: Codex
Task: NAUDIT-006
Risk: high
Status: REVIEW_REQUIRED

## Goal

证明 live startup 取消路径不仅会丢弃 pending future，还要求 adapter 的
`DataClient::connect` future 在被丢弃时释放临时资源、保持断开状态，并能明确后续重试边界。

## Changed Files

- `.agentflow/leases/NAUDIT-006.json`
- `.agentflow/state/task_status.json`
- `crates/common/src/clients/data.rs`
- `crates/live/src/node.rs`
- `docs/integrations/live_adapter_cancellation.md`
- `docs/rust-cutover/evidence/NAUDIT-006.md`

## Behavior Impact

- `LiveNode` 运行逻辑没有改变。
- `DataClient::connect` 的公开文档新增 cancellation contract。
- 新增 mock data client 测试，覆盖 startup stop 和 shutdown 两种取消路径。
- NAUDIT-004 的 `.agentflow` 状态从已合并后的 `REVIEW_REQUIRED` 收口为 `DONE`，不改业务代码。

## Public API Impact

No Rust function signature changed.

`DataClient::connect` 的语义文档更严格：live adapter 必须把 pending connect future 被丢弃视为失败的连接尝试，并清理临时资源。

## Migration Note Status

Added: `docs/integrations/live_adapter_cancellation.md`.

该文档明确 real adapters 还没有逐个完成 cancellation-safety 证明，后续每个 adapter 需要用 mock、fixture、sandbox 或 recorded harness 补证据。

## Validation

Commands were run with the project Rust toolchain from `scripts/ai/toolchain_env.sh`.

### Targeted smoke

```bash
source scripts/ai/toolchain_env.sh && cargo test -p nautilus-live test_data_client_connect_future_cleanup -- --nocapture
```

Result: passed.

Summary:

- `test_data_client_connect_future_cleanup_on_stop_request` passed.
- `test_data_client_connect_future_cleanup_on_shutdown_request` passed.
- The mock connect future acquired resources, was dropped by startup cancellation, and released the simulated resource and half-connected flags.

### Required validation

```bash
source scripts/ai/toolchain_env.sh && cargo test -p nautilus-live
```

Result: passed.

Summary:

- `nautilus-live` unit tests, integration tests, and doc tests passed.
- New cancellation tests passed inside `node::tests`.
- Existing `tests/stress.rs` still reports 2 ignored stress tests; this PR does not change that scope.

```bash
source scripts/ai/toolchain_env.sh && cargo check -p nautilus-live
```

Result: passed.

Summary:

- `nautilus-live` and its checked dependency chain completed successfully.

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

- Mock evidence proves the live-node cancellation boundary and adapter cleanup contract, but does not prove every real exchange adapter yet.
- Real adapter follow-up is listed in `docs/integrations/live_adapter_cancellation.md`.
- This task does not change dashboard/control API behavior.
- This task does not connect to real exchanges or use private credentials.

## Rollback Plan

Revert this PR. The rollback would remove the cancellation contract documentation and mock cleanup tests, but would not change current runtime behavior because this PR does not alter `LiveNode` startup logic.
