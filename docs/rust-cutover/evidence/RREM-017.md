# RREM-017 Evidence

Date: 2026-06-02
Executor: Codex
Task ID: RREM-017

## Goal

Remove PyO3/Python binding residue from `crates/model` while preserving the Rust
domain model, JSON custom-data registration, Arrow custom-data registration, and
FFI feature build.

## Plain Summary

这个 PR 清掉了 `crates/model` 里的 Python/PyO3 外壳。

大白话说，model crate 现在只保留 Rust 自己要用的模型类型、校验逻辑、
JSON 自定义数据注册和 Arrow/FFI 能力。以前为了 Python binding 放在类型
上的 `pyclass` 注解、stub 生成注解、Python custom-data wrapper、
Python extractor registry、Python-only 测试和相关说明都删掉了。

它没有改交易字段、价格数量计算、订单/事件/账户模型语义，也没有动 adapter、
indicator、common、core 等其他 crate 的残留。剩余 PyO3/Python 残留还在
其他 crate 里，需要继续分 slice 清。

## Files Changed

- `crates/model/**`
- `crates/model/README.md`
- `docs/rust-cutover/tasks/RREM-017.md`
- `docs/rust-cutover/evidence/RREM-017.md`
- `docs/rust-cutover/inventory/rust_crate_python_residue.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-016.json`
- `.agentflow/leases/RREM-017.json`

## Validation Commands

| Command | Result |
| --- | --- |
| `rg -n "pyo3\|pyo3_stub_gen\|PyO3\|feature = \"python\"\|python\|nautilus_trader" crates/model` | Pass; no matches. |
| `rg -n "cfg\\(feature = \"python\"\\)\|cfg_attr\\([^\\n]*python\|custom_data\\([^\\)]*(pyo3\|python)\|stub_module\|to_json_py\|to_pyobject\|PyExtractor\|PythonCustomDataWrapper\|register_python\|reconstruct_python" crates/model/src` | Pass; no matches. |
| `RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check -p nautilus-model --all-targets` | Pass; core dependency still emits expected non-model `python` cfg warnings. |
| `RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check -p nautilus-model --all-targets --features ffi` | Pass; core dependency still emits expected non-model `python` cfg warnings. |
| `RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check -p nautilus-model --all-targets --features arrow` | Pass; core dependency still emits expected non-model `python` cfg warnings. |
| `/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo fmt --check` | Pass |
| `scripts/ai/validate_agentflow_roles.py` | Pass |
| `git diff --check` | Pass |
| `scripts/ai/verify_fast.sh` | Pass |
| `scripts/ai/check_rust_only_runtime.sh` | Expected fail; remaining PyO3/Python residue is outside `crates/model`. |

## Toolchain Note

The default Homebrew toolchain is Rust 1.87.0 and is too old for this workspace.
Targeted cargo checks were run with the explicit Rust 1.95.0 toolchain path.

## Residual Rust-only Gate Blocker

`scripts/ai/check_rust_only_runtime.sh` still fails because active Rust product
paths outside `crates/model` retain PyO3/Python references.

Current non-model file-level count:

```text
325 files
```

Top remaining groups:

```text
162 crates/adapters
 41 crates/indicators
 27 crates/common
 14 crates/trading
 10 crates/persistence
  9 crates/network
  8 crates/execution
  8 crates/backtest
  7 crates/live
  7 crates/infrastructure
  6 crates/core
```

## Behavior Impact

No trading semantic change is intended. Rust model types, validation paths, JSON
custom-data registration, Arrow custom-data registration, and FFI feature checks
continue to compile. Python/PyO3 binding surfaces from `crates/model` are no
longer present.

## Review Status

Risk level: critical.

This task must stop at `REVIEW_REQUIRED`. Auto-merge is disabled.
