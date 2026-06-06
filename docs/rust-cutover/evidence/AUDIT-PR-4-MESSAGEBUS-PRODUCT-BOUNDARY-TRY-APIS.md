# AUDIT-PR-4 MessageBus product-boundary try APIs evidence

- Date: 2026-06-06
- Executor: Codex
- Task source: post-audit PR-4 follow-up
- Risk: medium

## Goal

给 MessageBus 的产品/用户输入边界补充 fallible API，让非法 topic 可以返回
`Err`，而不是只能走会 panic 的 infallible API。

本任务不是全仓 panic 清理，也不修改 MessageBus 的内部 invariant。旧 API 继续保留，
用于已验证的内部 topic 字符串。

## Files changed

- `crates/common/src/msgbus/core.rs`
- `crates/common/src/msgbus/api.rs`

## What changed

- 新增 `MessageBus::try_subscriptions_count(...)`。
- 新增 `MessageBus::try_has_subscribers(...)`。
- 新增 `try_subscriptions_count_any(...)`。
- 新增 `try_has_subscribers_any(...)`。
- 给旧的 `subscriptions_count(...)`、`has_subscribers(...)`、
  `subscriptions_count_any(...)` 补充 panic 文档，明确产品/用户输入边界应使用
  `try_*` API。
- 新增回归测试，验证非法 topic 通过 `try_*` API 返回错误，不触发 panic。

## What did not change

- 没有改变 typed route 与 `Any` route 的消息投递语义。
- 没有删除旧 API。
- 没有大范围清理所有 panic。
- 没有修改交易语义、adapter 行为、持久化格式或 feature flag 行为。

## Commands run

```bash
cargo test -p nautilus-common --lib test_try_subscriptions_count -- --nocapture
cargo test -p nautilus-common --lib test_route_separation -- --nocapture
cargo check -p nautilus-common
scripts/ai/verify_fast.sh
git diff --check
```

## Command results

- `cargo test -p nautilus-common --lib test_try_subscriptions_count -- --nocapture`
  passed: 2 tests passed.
- `cargo test -p nautilus-common --lib test_route_separation -- --nocapture`
  passed: 2 tests passed.
- `cargo check -p nautilus-common` passed.
- `scripts/ai/verify_fast.sh` passed. It is fast smoke only: toolchain and
  `cargo fmt --check` by default.
- `git diff --check` passed.

## Behavior impact

产品入口或外部输入如果需要检查 MessageBus subscription count，现在可以调用
`try_*` API 并处理错误，不需要依赖可能 panic 的路径。

内部已验证 topic 的旧调用方式保持不变。

## Public API impact

This is an additive Rust API change:

- `MessageBus::try_subscriptions_count`
- `MessageBus::try_has_subscribers`
- `try_subscriptions_count_any`
- `try_has_subscribers_any`

No existing public API was removed.

## Migration note

当 topic 来自用户输入、配置文件、CLI 参数、外部控制面或其他未验证边界时，应使用
`try_*` API。旧的 infallible API 仍适合内部常量、已验证 topic 或测试中的明确
invariant。

## Rollback plan

如需回滚，删除新增 `try_*` API 和对应测试即可。旧 API 和原有调用路径未删除，
因此回滚不会破坏现有调用者。
