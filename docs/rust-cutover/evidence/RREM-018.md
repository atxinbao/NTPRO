# RREM-018 Evidence

Date: 2026-06-02
Executor: Codex
Task ID: RREM-018
Risk: critical

## Plain Summary

这次把 `crates/adapters`、`crates/indicators`、`crates/common` 和
`crates/core` 里还残留的 PyO3/Python 绑定痕迹一起清掉了。大白话说，
这些目录以后不再对外声明“还能走 Python/PyO3 绑定”，也不再保留
`#[cfg(feature = "python")]`、`pyo3::pyclass`、stub-gen 注解和相关
fixture 文案。

没有改交易撮合、行情解析、订单路由、指标公式、clock/timer 语义或
adapter 行为。adapter fixture 没删，只把“以后再删 Python/PyO3”的旧审计
说明改成“RREM-018 已关闭这个 scoped removal gate”。

这是 critical removal 任务，PR 必须停在 `REVIEW_REQUIRED`，不启用
auto-merge。

## Changed Scope

- Removed PyO3/stub annotations and `feature = "python"` implementation blocks
  from:
  - `crates/adapters/**`
  - `crates/indicators/**`
  - `crates/common/**`
  - `crates/core/**`
- Rewrote scoped crate docs, README text, comments, test names, and adapter
  fixture notes that still described active Python/PyO3 product surfaces.
- Removed `Python.h` from scoped `cbindgen.toml` system include lists.
- Updated `docs/rust-cutover/inventory/rust_crate_python_residue.md`.
- Closed local RREM-017 task state after PR #113 was merged.

## Commands Run

```bash
rg -n -i "pyo3|pyo3_stub_gen|feature = \"python\"|cfg\\(feature = \"python\"\\)|cfg_attr\\([^\\n]*python|python|nautilus_pyo3|extension-module|custom_data\\([^\\)]*pyo3|stub_module|PyObject|PyAny|Py<" \
  crates/adapters crates/indicators crates/common crates/core
```

Result: no matches.

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
  -p nautilus-core --all-targets
```

Result: passed.

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check \
  -p nautilus-common --all-targets
```

Result: passed.

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check \
  -p nautilus-indicators --all-targets
```

Result: passed.

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check \
  --all-targets \
  -p nautilus-architect-ax -p nautilus-betfair -p nautilus-binance \
  -p nautilus-bitmex -p nautilus-blockchain -p nautilus-bybit \
  -p nautilus-coinbase -p nautilus-databento -p nautilus-deribit \
  -p nautilus-dydx -p nautilus-hyperliquid \
  -p nautilus-interactive-brokers -p nautilus-kraken -p nautilus-okx \
  -p nautilus-polymarket -p nautilus-sandbox -p nautilus-tardis
```

Result: passed in 27m32s.

Notes: The command emitted `unexpected cfg` warnings from non-scoped crates,
including `network`, `serialization`, `data`, `execution`, `portfolio`,
`infrastructure`, `trading`, `risk`, `system`, `backtest`, and `live`. Those
are outside RREM-018 allowed paths and are recorded in the updated inventory.

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
RREM-018 scope. Updated inventory now records 109 remaining files, with the
largest groups in `serialization`, `trading`, `execution`, `persistence`,
`network`, `backtest`, `live`, and `infrastructure`.

## Behavior Impact

- No intended trading behavior change.
- No intended adapter parser or execution behavior change.
- No intended indicator formula change.
- No intended clock/timer/msgbus behavior change.
- Public Rust APIs continue to compile through targeted checks.

## Follow-up

The next cleanup slices should target the remaining non-scoped crates in the
inventory. RREM-018 must stop at `REVIEW_REQUIRED` and wait for release
gatekeeper review before merge.
