# V02 release clippy cleanup evidence

- Date: 2026-06-06
- Executor: Codex
- Task source: v0.2 release-readiness blocker found by `scripts/ai/verify_release.sh`
- Risk: medium

## Goal

清理 `scripts/ai/verify_release.sh` 在 workspace clippy 阶段暴露的
`nautilus-cli` lint blocker，让 v0.2 tag/release 决策可以继续推进。

## Failure observed

`scripts/ai/verify_release.sh` 进入 `verify_full` 的 clippy 阶段后失败：

- `clippy::bool_to_int_with_if`
- `clippy::map_unwrap_or`
- `clippy::if_not_else`
- `clippy::needless_pass_by_value`
- `clippy::missing_errors_doc`
- `clippy::redundant_clone`

失败文件为 `crates/cli/src/supervisor.rs`。

## Files changed

- `crates/cli/src/supervisor.rs`

## What changed

- 用 `u64::from(...)` 替换布尔转整数的 `if`。
- 用 `map_or_else(...)` 替换 `map(...).unwrap_or_else(...)`。
- 调整 status artifact 存在性判断，满足 clippy 的 `if_not_else` 规则。
- `start_node_process(...)` 改为借用 `StartNodeRequest`，避免不必要的按值传参。
- 给 `write_node_metrics_artifact(...)` 补充 `# Errors` 文档。
- 移除测试中的冗余 `registry.clone()`。

## What did not change

- 没有改变 supervisor CLI 的用户可见命令。
- 没有改变 node start/stop/status 的行为语义。
- 没有修改 live runtime、adapter、交易语义或持久化格式。
- 没有扩大到其它 clippy 风格重构。

## Commands run

```bash
cargo fmt
cargo clippy -p nautilus-cli --lib --tests -- -D warnings
cargo test -p nautilus-cli --lib supervisor -- --nocapture
scripts/ai/verify_fast.sh
cargo clippy --workspace --lib --tests --features "arrow,ffi,high-precision,streaming,defi" -- -D warnings
git diff --check
```

## Command results

- `cargo clippy -p nautilus-cli --lib --tests -- -D warnings` passed.
- `cargo test -p nautilus-cli --lib supervisor -- --nocapture` passed:
  12 tests passed.
- `scripts/ai/verify_fast.sh` passed. It is fast smoke only.
- `cargo clippy --workspace --lib --tests --features "arrow,ffi,high-precision,streaming,defi" -- -D warnings`
  passed.
- `git diff --check` passed.

## Behavior impact

无行为变更。该 PR 只清理 release gate 暴露的 clippy blocker，让相同代码路径在
`-D warnings` 下可通过。

## Public API impact

`SupervisorRegistryStore::start_node_process(...)` 从按值接收
`StartNodeRequest` 改为按引用接收。该接口位于 CLI crate 的 supervisor 实现内，
当前改动同步更新了本 crate 内调用点。

## Migration note

不需要用户迁移说明。该修复不改变 CLI 命令、输出格式或 artifacts 结构。

## Rollback plan

如需回滚，撤销本 PR 即可。回滚后 release gate 会重新暴露同一批 clippy blocker。
