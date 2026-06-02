# RREM-020 Evidence

Date: 2026-06-02
Executor: Codex
Task ID: RREM-020
Risk: critical

## Plain Summary

本次清理 runtime-facing Rust crate 里剩余的一批 PyO3/Python 痕迹，范围是
`execution`、`backtest`、`live`、`trading`、`risk`、`portfolio` 和 `data`。

大白话说，这些运行时 crate 以后不再保留 `feature = "python"`、
`pyo3::pyclass`、stub 生成注解、Python actor 桥接字段、Python-only 示例导入，
以及继续暗示 Python 绑定仍是产品面的说明。

不会改交易算法、回测撮合、live 生命周期、订单执行、风控拒单、组合/PnL、
data engine 行为或 adapter 行为。本任务只清掉 Rust-only 产品面不再需要的
Python/PyO3 外壳。

这是 critical removal 任务，PR 必须停在 `REVIEW_REQUIRED`，不启用 auto-merge。

## Changed Scope

- Removed PyO3/stub annotations from runtime-facing config, node, model, and
  result structs in `execution`, `backtest`, `live`, `trading`, `risk`,
  `portfolio`, and `data`.
- Removed Python-only `pymethods` impl blocks from Rust trading example
  configuration structs.
- Removed the unused Python actor field from `LiveNode`.
- Replaced `backtest` accumulator tests with Rust-native callback tests.
- Removed old Python-facing usage sections from scoped trading example READMEs.
- Rewrote scoped comments and crate feature docs to Rust-only or legacy
  reference wording.
- Removed `Python.h` from the scoped `backtest` cbindgen config.
- Updated the Rust crate Python/PyO3 residue inventory from 81 remaining files
  to 31 remaining non-scoped files.
- Closed local RREM-019 repo state after PR #115 was merged.

## Commands Run

```bash
rg -n -i "pyo3|pyo3_stub_gen|feature = \"python\"|cfg\\(feature = \"python\"\\)|cfg_attr\\([^\\n]*python|python|nautilus_pyo3|extension-module|custom_data\\([^\\)]*pyo3|stub_module|PyObject|PyAny|Py<" \
  crates/execution crates/backtest crates/live crates/trading crates/risk crates/portfolio crates/data
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
python3 scripts/ai/validate_agentflow_roles.py
```

Result: passed.

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check \
  -p nautilus-execution --all-targets
```

Result: passed in 7m25s.

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check \
  -p nautilus-backtest --all-targets
```

Result: passed in 13m17s.

Notes: This check surfaced non-scoped `unexpected cfg` warnings from
`crates/system` and `crates/persistence`. Those crates are outside RREM-020 and
are recorded in the follow-up inventory.

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check \
  -p nautilus-live --all-targets
```

Result: passed in 3m57s.

Notes: This check surfaced the same non-scoped `crates/system` warning.

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check \
  -p nautilus-trading --all-targets
```

Result: passed in 1m30s.

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check \
  -p nautilus-risk --all-targets
```

Result: passed in 37.34s.

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check \
  -p nautilus-portfolio --all-targets
```

Result: passed in 2m23s.

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check \
  -p nautilus-data --all-targets
```

Result: passed in 2m35s.

```bash
scripts/ai/verify_fast.sh
```

Result: passed.

```bash
scripts/ai/check_rust_only_runtime.sh
```

Result: expected failure.

Reason: Rust-only runtime still has 31 non-scoped Rust crate files with
Python/PyO3 residue in `crates/persistence`, `crates/infrastructure`,
`crates/model`, `crates/system`, `crates/plugin`, `crates/testkit`, and
`crates/cryptography`.

## Behavior Impact

- No intended backtest, live, trading, data, execution, risk, or portfolio
  runtime behavior change.
- No intended order matching, position lifecycle, risk rejection, PnL,
  accounting, event ordering, data engine, live node, or adapter behavior
  change.
- Rust-native accumulator tests now cover the ordering behavior that was
  previously tested only behind a Python/PyO3 test gate.
- Public Rust APIs for the scoped crates continue to compile through targeted
  checks.

## Follow-up

Remaining cleanup should continue with the non-scoped support crates recorded
in the inventory: `persistence`, `infrastructure`, `plugin`, `testkit`,
`cryptography`, residual `model` comments/docs, and `system`.

RREM-020 must stop at `REVIEW_REQUIRED` and wait for release gatekeeper review
before merge.

PR: https://github.com/atxinbao/NTPRO/pull/116
