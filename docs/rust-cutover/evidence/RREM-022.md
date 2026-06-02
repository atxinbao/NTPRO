# RREM-022 Evidence

Date: 2026-06-02
Executor: Codex
Task ID: RREM-022
Risk: critical

## Plain Summary

本次清理 Rust crate 里最后一批 Python/PyO3 残留，范围是
`crates/model` 和 `crates/system`。

大白话说，前面几轮已经清掉大部分 Python/PyO3 代码和文档，这次是收尾：
去掉 model C 头文件配置里的 `Python.h`，把 FFI 注释从“Python string”
改成普通 C 字符串说明，把 system crate 文档里的 Python feature 说明删掉，
并把原来只在 `feature = "python"` 下编译的 system 测试改成普通 Rust 测试。

不会改 enum 值、FFI 函数名、交易逻辑、kernel 行为或 workspace 结构。

这是 critical removal 任务，PR 必须停在 `REVIEW_REQUIRED`，不启用 auto-merge。

## PR

PR: https://github.com/atxinbao/NTPRO/pull/118
Status: REVIEW_REQUIRED; auto-merge disabled.

## Changed Scope

- Removed `Python.h` from `crates/model/cbindgen.toml`.
- Reworded model C FFI enum comments from "Python string" to "C string" without
  changing any exported FFI function names.
- Removed stale Python mixed-operation docs from model value type docs.
- Reworded model account/greeks comments that referenced Python compatibility.
- Removed Python/PyO3/extension-module feature docs from `crates/system`.
- Converted system builder/kernel tests from Python-feature-gated tests to
  normal Rust tests.
- Updated the Rust crate Python residue inventory to record zero remaining Rust
  crate product-path hits.

## Commands Run

```bash
rg -n -i "pyo3|pyo3_stub_gen|feature = \"python\"|cfg\\(feature = \"python\"\\)|cfg_attr\\([^\\n]*python|python|nautilus_pyo3|extension-module|custom_data\\([^\\)]*pyo3|stub_module|PyObject|PyAny|Py<" \
  crates --glob "*.rs" --glob "*.toml" --glob "*.md"
cargo fmt
cargo fmt --check
git diff --check
python3 scripts/ai/validate_agentflow_roles.py
scripts/ai/check_rust_only_runtime.sh
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check -p nautilus-model --all-targets
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check -p nautilus-system --all-targets
scripts/ai/verify_fast.sh
```

Results:

- Full Rust crate residue scan: no matches.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `scripts/ai/check_rust_only_runtime.sh`: passed.
- `nautilus-model --all-targets`: passed.
- `nautilus-system --all-targets`: passed.
- `scripts/ai/verify_fast.sh`: passed.

## Behavior Impact

- No intended trading behavior change.
- No enum value, FFI function name, or C ABI layout change.
- Model C header generation no longer lists `Python.h`.
- System builder/kernel tests now compile as ordinary Rust tests instead of
  requiring a Python feature gate.
- Rust-only runtime gate is now green.

## Follow-up

- Do not run `RREL-008` automatically. Release completion still needs
  release-gate review and final signoff.
