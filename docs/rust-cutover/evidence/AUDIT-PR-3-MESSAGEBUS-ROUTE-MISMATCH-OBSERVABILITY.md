# AUDIT-PR-3 - MessageBus Route Mismatch Observability Evidence

Date: 2026-06-06
Executor: Codex
Branch: `codex/msgbus-route-mismatch-observability`

## Task

审计修复项 PR-3：增加 MessageBus typed / Any route mismatch
可观察性。RCORE-008 已经有测试证明 typed 和 Any 路由不会互相投递，本任务不
改变该行为，只增加计数和 debug 日志，方便发现误用。

## Goal

- typed publish 时，如果同 topic 还有 Any 订阅者，记录 mismatch。
- Any publish 时，如果 payload 类型和 topic 还有 typed 订阅者，记录
  mismatch。
- 保持 typed / Any 路由分离，不做跨路由投递。
- 保留 RCORE-008 route separation 行为。

## Files Changed

- `crates/common/src/msgbus/core.rs`
- `crates/common/src/msgbus/api.rs`
- `crates/common/src/msgbus/mod.rs`
- `docs/rust-cutover/evidence/AUDIT-PR-3-MESSAGEBUS-ROUTE-MISMATCH-OBSERVABILITY.md`

## Change Summary

- Added public `RouteMismatchStats`.
- Added `route_mismatch_stats()` for the current thread-local bus.
- Added debug log and counters for:
  - typed publishes with matching Any subscribers;
  - Any publishes with matching typed subscribers.
- Added tests proving counters increment while the wrong-route subscriber still
  does not receive the message.

## Commands Run

Initial invalid attempt:

```bash
cargo test -p nautilus-common --lib test_route_mismatch_stats -- --nocapture
cargo test -p nautilus-common --lib test_route_separation -- --nocapture
```

Result: invalid run. These were started in parallel and contended for Cargo's
artifact lock. The stale cargo/rustc/lsof processes were stopped, then the
tests were rerun serially.

Serial targeted tests:

```bash
cargo test -p nautilus-common --lib test_route_mismatch_stats -- --nocapture
```

Result: passed. 2 tests passed, 0 failed.

```bash
cargo test -p nautilus-common --lib test_route_separation -- --nocapture
```

Result: passed. 2 tests passed, 0 failed.

Crate check and fast smoke:

```bash
cargo check -p nautilus-common
```

Result: passed.

```bash
scripts/ai/verify_fast.sh
```

Result: passed. Toolchain smoke and `cargo fmt --check` passed. The script
reported that workspace cargo check and clippy are intentionally skipped in
default fast-smoke mode.

```bash
git diff --check
```

Result: passed.

## Behavior Impact

Message delivery semantics are unchanged:

- typed publishes still deliver only to typed handlers;
- Any publishes still deliver only to Any handlers;
- mismatch counters/logs make opposite-route subscribers visible.

This affects observability only, not trading behavior or adapter behavior.

## Public API Impact

Additive API:

- `RouteMismatchStats`
- `route_mismatch_stats()`
- `MessageBus::route_mismatch_stats()`

No existing public API signature was changed.

## Migration Note

No migration note is required. Existing users do not need to change code. Users
who diagnose MessageBus route misuse can now read the counters.

## Rollback Plan

Revert this PR to remove `RouteMismatchStats`, the `route_mismatch_stats`
accessors, the counter updates, and the two tests. No persisted data, trading
state, or schema rollback is required.
