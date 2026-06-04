# GH-158 - Todo panic explicit error evidence

Date: 2026-06-05
Executor: Codex

## Task

- GitHub issue: <https://github.com/atxinbao/NTPRO/issues/158>
- Branch: `codex/audit-explicit-not-implemented-errors`
- Local task file: not present; this is a GitHub audit issue execution.
- Owner role: Rust Core Runtime Agent / Adapter & Integration Agent
- Review role: Verification & Release Gatekeeper
- Risk level: Medium

## Plain Chinese summary

这次不是补完整 SQL、Redis 或 blockchain 功能，也没有改交易逻辑。

改动的重点是：以前一些产品或 adapter 可能走到的 Rust 路径里还有
`todo!()`，一旦被调用就会直接 panic。现在这些点会返回明确的
“not implemented / unsupported” 错误，调用方能拿到失败原因，不会被隐藏成
运行时崩溃。

剩余的 `todo!()` 命中已经分类：主要是 analysis 测试 mock 和文档注释，不是这次
产品运行面要修的 panic blocker。

## Goal

Replace product-reachable `todo!()` / `unimplemented!()` panic paths with
explicit unsupported or not-implemented errors, and record the remaining
classification.

## Files changed

- `crates/adapters/blockchain/src/execution/client.rs`
- `crates/adapters/blockchain/src/rpc/mod.rs`
- `crates/common/src/cache/mod.rs`
- `crates/infrastructure/src/redis/cache.rs`
- `crates/infrastructure/src/sql/cache.rs`
- `crates/infrastructure/src/sql/models/instruments.rs`
- `crates/infrastructure/src/sql/models/orders.rs`
- `docs/rust-cutover/scope/GH-158-TODO-PANIC-SCOPE.md`
- `docs/rust-cutover/evidence/GH-158-TODO-PANIC-EXPLICIT-ERRORS.md`

## Commands run

```bash
rg -n '\b(todo!|unimplemented!)\s*\(' crates --glob '*.rs' -S
cargo fmt --all
source scripts/ai/toolchain_env.sh && CARGO_BUILD_JOBS=2 cargo check -p nautilus-infrastructure
source scripts/ai/toolchain_env.sh && CARGO_BUILD_JOBS=2 cargo check -p nautilus-blockchain
source scripts/ai/toolchain_env.sh && CARGO_BUILD_JOBS=2 cargo check -p nautilus-common -p nautilus-infrastructure -p nautilus-blockchain
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_cython_removed.sh
scripts/ai/verify_fast.sh
git diff --check
```

## Result summary

- Product/adapter reachable SQL, Redis, and blockchain `todo!()` panic paths
  were replaced by explicit errors.
- `cargo check -p nautilus-infrastructure` passed.
- `cargo check -p nautilus-blockchain` passed.
- Combined check for `nautilus-common`, `nautilus-infrastructure`, and
  `nautilus-blockchain` passed.
- `scripts/ai/check_rust_only_runtime.sh` passed.
- `scripts/ai/check_cython_removed.sh` passed.
- `scripts/ai/verify_fast.sh` passed.
- `git diff --check` passed.

## Remaining `todo!()` / `unimplemented!()` classification

- `crates/analysis/src/analyzer.rs`: test-local `MockAccount` helper methods.
- `crates/persistence/src/backend/catalog.rs`: doctest placeholder comments.
- `crates/execution/src/order_manager/manager.rs`: historical doc comment.

## Behavior impact

Unsupported SQL/Redis/blockchain operations now fail with explicit errors
instead of panicking. Supported implemented paths were not changed.

## Public API impact

No public API or trait signature changed.

## Migration note status

No migration note required. This is an error-handling hardening change for
previously unimplemented paths.

## Rollback plan

Revert this PR. That would restore the previous panic behavior for the affected
unsupported paths, with no schema or runtime state migration involved.
