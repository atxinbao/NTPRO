# RREM-019 Evidence

Date: 2026-06-02
Executor: Codex
Task ID: RREM-019
Risk: critical

## Plain Summary

本次清理 `crates/serialization` 和 `crates/network` 里残留的
PyO3/Python 痕迹。大白话说，这两个底层 crate 以后不再保留
`feature = "python"`、`pyo3::pyclass`、stub 生成注解、PyErr 转换、
Python-only socket 测试模块，以及继续暗示 Python 绑定仍是产品面的文档。

没有改 Arrow 字段格式、SBE 解码、HTTP 请求逻辑、WebSocket/TCP 重连逻辑、
限流逻辑或 adapter 行为。只是把已经不属于 Rust-only 产品面的 Python/PyO3
外壳清掉，并保留 Rust 原生测试覆盖。

这是 critical removal 任务，PR 必须停在 `REVIEW_REQUIRED`，不启用
auto-merge。

## Changed Scope

- Removed PyO3/stub annotations and `feature = "python"` code from
  `crates/network`.
- Removed PyO3 import/error conversion residue from `crates/serialization`.
- Removed the Python-only socket test module; Rust-native socket tests remain.
- Rewrote scoped comments/docs that still described active Python/PyO3 product
  surfaces.
- Updated the Rust crate Python/PyO3 residue inventory.
- Closed local RREM-018 repo state after PR #114 was merged.

## Commands Run

```bash
rg -n -i "pyo3|pyo3_stub_gen|feature = \"python\"|cfg\\(feature = \"python\"\\)|cfg_attr\\([^\\n]*python|python|nautilus_pyo3|extension-module|custom_data\\([^\\)]*pyo3|stub_module|PyObject|PyAny|Py<" \
  crates/serialization crates/network
```

Result: passed; no matches.

```bash
/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo fmt --check
```

Result: passed.

```bash
git diff --check
```

Result: passed.

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check \
  -p nautilus-serialization --all-targets
```

Result: passed in 1m52s.

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check \
  -p nautilus-network --all-targets
```

Result: passed in 10m59s.

Notes: This command no longer emits `feature = "python"` warnings from
`crates/serialization` or `crates/network`.

```bash
python3 scripts/ai/validate_agentflow_roles.py
```

Result: passed.

```bash
scripts/ai/verify_fast.sh
```

Result: passed.

```bash
scripts/ai/check_rust_only_runtime.sh
```

Result: expected failure.

Reason: Rust-only runtime still has remaining Python/PyO3 residue outside the
RREM-019 scope. Updated inventory now records 81 remaining files, with the
largest groups in `trading`, `execution`, `persistence`, `backtest`, `live`,
and `infrastructure`.

## Behavior Impact

- No intended Arrow schema or serialization behavior change.
- No intended HTTP, WebSocket, TCP socket, rate limiter, or retry behavior
  change.
- No intended adapter behavior change.
- Public Rust APIs continue to compile through targeted checks.

## Follow-up

Remaining cleanup should continue with runtime-facing crates outside this slice.
RREM-019 must stop at `REVIEW_REQUIRED` and wait for release gatekeeper review
before merge.
