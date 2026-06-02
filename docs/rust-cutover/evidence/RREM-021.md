# RREM-021 Evidence

Date: 2026-06-02
Executor: Codex
Task ID: RREM-021
Risk: critical

## Plain Summary

本次清理 support Rust crate 里剩余的一批 PyO3/Python 痕迹，范围是
`persistence`、`infrastructure`、`plugin`、`testkit` 和 `cryptography`。

大白话说，这些支撑模块以后不再保留 `feature = "python"`、
`pyo3::pyclass`、stub 生成注解、`custom_data(pyo3)` 宏入口、Python-only
测试，以及继续暗示 Python 绑定仍是产品面的说明。

不会改 catalog 存储格式、Arrow/Parquet 格式、数据库行为、Redis 行为、
plug-in ABI 行为或加密算法。本任务只清掉 Rust-only 产品面不再需要的
Python/PyO3 外壳。

这是 critical removal 任务，PR 必须停在 `REVIEW_REQUIRED`，不启用 auto-merge。

## PR

PR: https://github.com/atxinbao/NTPRO/pull/117
Status: REVIEW_REQUIRED; auto-merge disabled.

## Changed Scope

- Removed PyO3/stub generation support from `crates/persistence/macros`:
  - `#[custom_data(pyo3)]`, `#[custom_data(python)]`, and `stub_module` are no
    longer accepted options.
  - Generated `pyo3::pyclass`, `pyo3::pymethods`, pyo3-stub-gen annotations,
    PyArrow bridge methods, Python constructors/getters, and Python conversion
    helpers were removed.
  - Rust-only `#[custom_data]`, `no_arrow`, `no_display`, JSON field support,
    Arrow encode/decode generation, catalog path conversion, and `Data`
    conversions remain.
- Converted persistence test custom data fixtures from `#[custom_data(pyo3)]`
  to Rust-only `#[custom_data]`.
- Removed scoped PyO3/Python attributes, feature-gated blocks, and binding docs
  from `persistence`, `infrastructure`, `plugin`, `testkit`, and
  `cryptography`.
- Removed the Python-gated custom data decode fallback. Unknown custom data now
  requires Rust registration with `ensure_custom_data_registered`.
- Updated `docs/rust-cutover/inventory/rust_crate_python_residue.md`; remaining
  Rust crate residue is now limited to `crates/model` and `crates/system`.

## Commands Run

```bash
cargo fmt
cargo fmt --check
git diff --check
python3 scripts/ai/validate_agentflow_roles.py
rg -n -i "pyo3|pyo3_stub_gen|feature = \"python\"|cfg\\(feature = \"python\"\\)|cfg_attr\\([^\\n]*python|python|nautilus_pyo3|extension-module|custom_data\\([^\\)]*pyo3|stub_module|PyObject|PyAny|Py<" \
  crates/persistence crates/infrastructure crates/plugin crates/testkit crates/cryptography
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check -p nautilus-persistence-macros --all-targets
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check -p nautilus-persistence --all-targets
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check -p nautilus-infrastructure --all-targets
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check -p nautilus-plugin --all-targets
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check -p nautilus-testkit --all-targets
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo check -p nautilus-cryptography --all-targets
scripts/ai/verify_fast.sh
scripts/ai/check_rust_only_runtime.sh
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc \
  /Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo test -p nautilus-persistence-macros --lib
```

Results:

- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- Scoped residue scan for this task: no matches.
- `nautilus-persistence-macros --all-targets`: passed.
- `nautilus-persistence --all-targets`: passed.
- `nautilus-infrastructure --all-targets`: passed.
- `nautilus-plugin --all-targets`: passed.
- `nautilus-testkit --all-targets`: passed.
- `nautilus-cryptography --all-targets`: passed.
- `scripts/ai/verify_fast.sh`: passed.
- `cargo test -p nautilus-persistence-macros --lib`: passed, 5 tests.
- `scripts/ai/check_rust_only_runtime.sh`: expected failure on non-scoped
  residue. The first printed blocker was `crates/system/src/lib.rs`; the
  broader scan shows remaining files only under `crates/model` and
  `crates/system`.
- Optional targeted persistence lib test
  `cargo test -p nautilus-persistence test_write_custom_data_round_trip --lib`
  was stopped after it spent more than eight minutes compiling/linking without
  output and no CPU activity. The required `nautilus-persistence --all-targets`
  check had already passed.

## Behavior Impact

- Rust custom data macro use remains available as `#[custom_data]`.
- Python/PyO3 binding generation for custom data is removed.
- Custom data decoding no longer has a Python feature fallback; Rust custom data
  types must be registered before reading unknown custom data from persistence.
- No intended change to catalog storage layout, Arrow/Parquet schema, Redis,
  PostgreSQL, plug-in ABI, testkit behavior, or cryptographic algorithms.

## Follow-up

- Clean remaining `crates/model` residue, including FFI enum comments and
  cbindgen `Python.h`.
- Clean remaining `crates/system` residue, including crate-level feature docs
  and test-gated Python cfg.
- Re-run `scripts/ai/check_rust_only_runtime.sh` after those slices.
