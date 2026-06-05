# GH-159 Live Startup Cancellation Evidence

- Date: 2026-06-05
- Executor: Codex
- GitHub issue: #159 `[Audit] Harden live node startup stop/shutdown responsiveness`
- Branch: `codex/audit-live-startup-cancel`
- Formal task file: not present in the Shrimp queue; this is a GitHub audit
  issue execution task.
- Risk level: Medium to High.
- Gate status: actual live lifecycle behavior changed, so this PR must stop at
  human review and must not be auto-merged.

## Plain Chinese Summary

这次处理的是 live node 启动时“卡在 data client 连接阶段不好停”的问题。

以前 `LiveNode::run()` 在连接 data clients 时，只等连接流程自己结束。如果这个连接流程长时间不返回，外部 stop、shutdown flag 或 Ctrl-C 在这个阶段就不够及时。现在 data client 启动阶段会边等待连接、边监听停止信号：收到 stop、shutdown 或 Ctrl-C 后，会走已有的 startup abort 流程，断开客户端并退出 event loop。

手动 `LiveNode::start()` 路径也补了 stop/shutdown 检查，但没有额外接 Ctrl-C，因为 Ctrl-C 主要属于 `run()` 的事件循环职责。

这次没有实现 dashboard UI、control API、真实 adapter 连接行为，也没有修改交易语义。execution client 连接阶段保持原有逻辑。

## Goal

Make live node startup more responsive to stop and shutdown controls during the
data-client connection phase, while preserving the existing startup buffering
and shutdown flow.

## Scope

Changed:

- `crates/live/src/node.rs`
- `docs/rust-cutover/evidence/GH-159-LIVE-STARTUP-CANCELLATION.md`

Not changed:

- dashboard UI;
- control API endpoints;
- live adapter connection implementations;
- execution-client startup behavior;
- trading semantics;
- public Rust API;
- release tags or GitHub Releases.

## Implementation Notes

- Added `EngineConnectionStatus::InterruptReceived` so startup can classify a
  Ctrl-C interrupt separately from stop and shutdown requests.
- Added `startup_control_status()` to centralize stop handle and shutdown flag
  checks.
- Added `await_startup_future()` for manual startup futures that should abort
  when stop or shutdown is requested.
- Added `drive_data_connect_with_event_buffering()` for the `run()` data-client
  phase. It keeps the existing event buffering behavior while also watching:
  - `LiveNodeHandle::stop()`;
  - the kernel shutdown flag;
  - `tokio::signal::ctrl_c()`.
- On abort, `LiveNode::run()` now calls the existing `abort_startup()` path,
  drains startup channels, logs `Event loop stopped`, and returns.

## Dashboard / Control Precondition

This change improves the startup cancellation checkpoint needed before a
dashboard or control API can safely manage a live node. It does not provide the
control endpoint itself. Future dashboard/control work still needs an explicit
runtime status model and command boundary before exposing live-node control to
users.

## Validation

| Command | Result | Notes |
|---------|--------|-------|
| `source scripts/ai/toolchain_env.sh && CARGO_BUILD_JOBS=2 cargo test -p nautilus-live --lib startup -- --nocapture` | passed | 9 tests passed, including new stop/shutdown startup future tests and existing startup abort smoke. |
| `source scripts/ai/toolchain_env.sh && CARGO_BUILD_JOBS=2 cargo test -p nautilus-live --test node test_handle_stop_triggers_graceful_shutdown -- --nocapture` | passed | 1 integration smoke passed for handle-driven shutdown. |
| `source scripts/ai/toolchain_env.sh && CARGO_BUILD_JOBS=2 cargo check -p nautilus-live` | passed | Finished `dev` profile in 2m 39s. |
| `scripts/ai/check_rust_only_runtime.sh` | passed | Reported `rust-only-runtime: ok`. |
| `scripts/ai/check_cython_removed.sh` | passed | Reported `cython-removed: ok`. |
| `scripts/ai/verify_fast.sh` | passed | Fast smoke passed; script output confirms workspace cargo check and clippy are skipped by default. |
| `git diff --check` | passed | No whitespace errors. |

## Behavior Impact

Live startup can now abort the data-client connection phase when stop,
shutdown, or Ctrl-C is observed. The abort uses existing disconnect and channel
drain paths instead of adding a new shutdown mechanism.

The new cancellation point drops the pending data-client connection future when
startup is aborted. This is the normal async cancellation mechanism, but real
adapter futures still need to be cancellation-safe in their own implementation.

## Public API Impact

No public Rust API changed. The added helper functions and enum variant are
private to `crates/live/src/node.rs`.

## Migration Note Status

No migration note is required. This is runtime hardening inside the existing
live-node startup flow and does not change CLI syntax, exported Rust API,
configuration schema, or adapter API.

## Remaining Risk

- Real network adapter startup was not exercised; tests cover helper behavior,
  startup abort flow, and handle-stop smoke.
- Ctrl-C handling was added to the `run()` event-loop startup phase, not the
  manual `start()` helper path.
- Execution-client startup retains its current behavior and was not expanded in
  this PR.
- Because this modifies live lifecycle behavior, human review is required
  before merge.

## Rollback Plan

Revert this PR to restore the prior startup flow. No data migration, release
rollback, or API rollback is required.
