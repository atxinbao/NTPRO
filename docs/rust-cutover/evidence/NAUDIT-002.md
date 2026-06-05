# NAUDIT-002 CLI Capability Matrix Evidence

Date: 2026-06-05
Executor: Codex
Task ID: NAUDIT-002
Risk: medium

## 中文摘要

这次修的是“CLI 到底能干什么”的口径问题，不是新增交易能力。

主要变化：

- 新增 CLI 能力矩阵，把命令分成 `implemented`、`simulated_demo`、
  `metadata_only`、`deferred` 四类。
- 把 `sandbox run` 的输出从容易误解的 `node_started=true` 改成
  `live_node_started=false`，明确说明它只是 simulated demo，没有启动真实
  `LiveNode`。
- 同步更新 CLI help 文案和产品文档，避免把 stub 或模拟流程写成真实 runtime。

## Scope

Changed:

- `crates/cli/src/sandbox.rs`
- `crates/cli/src/opt.rs`
- `docs/rust-cutover/product/CLI_CAPABILITY_MATRIX.md`
- `docs/rust-cutover/product/CLI_HELP_CONTRACT.md`
- `docs/rust-cutover/product/RUST_CLI_CONTRACT.md`
- `docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md`
- `docs/rust-cutover/product/RUST_API_ENTRYPOINTS.md`
- `docs/rust-cutover/product/RUST_PRODUCT_SURFACE_REPORT.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/NAUDIT-002.json`

Not changed:

- No backtest runtime wiring.
- No live runtime wiring.
- No dashboard UI.
- No control API.
- No adapter behavior.
- No trading semantics.

## Behavior Impact

`nautilus sandbox run` still writes deterministic demo artifacts, but its
artifact wording is more honest:

- removed `node_started=true`;
- removed `node_stopped=true`;
- added `live_node_started=false`;
- added `live_node_stopped=false`;
- added `simulated_lifecycle_status=completed`;
- renamed simulated event rows from `event=node_start/node_stop` to
  `event=simulate_node_start/simulate_node_stop`.

This is an owner-visible artifact wording change only. It does not start or
stop a real `LiveNode`.

## Public API Impact

No Rust public API changed. CLI artifact text changed for `sandbox run` to avoid
overstating runtime capability.

## Validation Commands

First attempt without sourcing the project toolchain failed because the shell
resolved rustc `1.87.0`, while NTPRO requires rustc `1.95.0`. Commands were
rerun with:

```bash
source scripts/ai/toolchain_env.sh
```

Final commands:

```bash
cargo fmt -p nautilus-cli
source scripts/ai/toolchain_env.sh && cargo test -p nautilus-cli
source scripts/ai/toolchain_env.sh && cargo check -p nautilus-cli
git diff --check
scripts/ai/validate_agentflow_roles.py
scripts/ai/verify_fast.sh
rg -n "node_started=true|node_stopped=true|event=node_start|event=node_stop|partially supported|sandbox product commands are not implemented yet|Runtime execution for backtest, sandbox, live, data, and config commands still" crates/cli docs/rust-cutover/product
```

## Results

- `cargo test -p nautilus-cli`: passed with rustc `1.95.0`; 42 tests passed.
- `cargo check -p nautilus-cli`: passed with rustc `1.95.0`.
- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`; as expected,
  it skipped workspace `cargo check`, clippy, release gate, and golden trace
  gate.
- Old misleading wording search: no matches.

## Migration Note Status

No product migration note is required. The changed `sandbox run` artifact fields
are documented by this evidence and the CLI capability matrix.

## Rollback Plan

Revert the NAUDIT-002 PR to restore the previous sandbox artifact wording and
remove the capability matrix updates.
