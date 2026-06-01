# RREM-016 Evidence

Date: 2026-06-02
Executor: Codex
Task ID: RREM-016

## Goal

Remove Rust crate Cython build residue and clear the no-longer-declared
`crates/analysis` PyO3/Python annotations, while recording the remaining broader
Rust crate Python/PyO3 cleanup surface.

## Plain Summary

这个 PR 继续清理 Rust crate 里的历史 Python/Cython 痕迹。

它做了两件事：第一，Rust 的 build script 不再生成 Cython 用的 `.pxd`
文件，也删掉了对应的 `cbindgen_cython.toml` 配置和 `pyproject.toml` 里的
`cython` 依赖；第二，`crates/analysis` 这个 Rust crate 本身已经没有
`python` feature，所以把里面残留的 PyO3 注解删掉。

它还把 `ffi` 打开时生成 C header 的路径改到 Cargo 的 `OUT_DIR`，避免
build script 重新创建已经删除的 `nautilus_trader/` Python 包目录。

它没有一次性删除其他 crate 里的所有 PyO3/Python 字符串，因为剩余范围很大，
还分布在 model、adapter、common、indicator 等模块里。那些需要后续按 crate
分批清理，避免一个 PR 改太多、review 不清楚。

## Files Changed

- `Cargo.toml`
- `pyproject.toml`
- `crates/core/build.rs`
- `crates/common/build.rs`
- `crates/backtest/build.rs`
- `crates/model/build.rs`
- `crates/*/cbindgen_cython.toml`
- `crates/analysis/**`
- `docs/developer_guide/rust.md`
- `docs/rust-cutover/inventory/core_model_value_types.md`
- Rust source comment-only cleanup for stale Cython wording
- `docs/rust-cutover/tasks/RREM-016.md`
- `docs/rust-cutover/evidence/RREM-016.md`
- `docs/rust-cutover/inventory/rust_crate_python_residue.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-016.json`

## Validation Commands

| Command | Result |
| --- | --- |
| `rg -n "Cython|cythonize|\\.pyx|\\.pxd|cbindgen_cython|nautilus_trader/core/rust|cython" crates Cargo.toml Makefile pyproject.toml --glob '!docs/**'` | Pass; no matches. |
| `rg -n "nautilus_trader/core/includes|nautilus_trader/core/rust" crates Cargo.toml Makefile pyproject.toml docs/developer_guide docs/rust-cutover/inventory/core_model_value_types.md` | Pass; no matches. |
| `rg -n "pyo3|pyo3_stub_gen|PyO3|feature = \"python\"|python" crates/analysis` | Pass; no matches. |
| `scripts/ai/check_cython_removed.sh` | Pass |
| `scripts/ai/check_no_cython_runtime.sh` | Pass |
| `cargo metadata --format-version=1 --no-deps` | Pass; `nautilus-analysis` present. |
| `cargo fmt --check` | Pass |
| `scripts/ai/validate_agentflow_roles.py` | Pass |
| `git diff --check` | Pass |
| `scripts/ai/verify_fast.sh` | Pass |
| `cargo check -p nautilus-analysis --all-targets` with Rust 1.95.0 | Pass; dependency warnings show remaining core/model Python cfg residue. |
| `cargo check -p nautilus-core --all-targets --features ffi` with Rust 1.95.0 | Pass; generated header path no longer recreates `nautilus_trader/`. |
| `cargo check -p nautilus-common --all-targets --features ffi` with Rust 1.95.0 | Pass; generated header path no longer recreates `nautilus_trader/`. |
| `cargo check -p nautilus-backtest --all-targets --features ffi` with Rust 1.95.0 | Pass; generated header path no longer recreates `nautilus_trader/`. |
| `cargo check -p nautilus-model --all-targets --features ffi` with Rust 1.95.0 | Pass; generated header path no longer recreates `nautilus_trader/`. |
| `scripts/ai/check_rust_only_runtime.sh` | Expected fail; 1,509-line log still reports remaining PyO3/Python residue outside this task scope. |

Rust 1.95.0 was selected explicitly with:

```bash
RUSTC="$(rustup which rustc --toolchain 1.95.0)" rustup run 1.95.0 cargo check ...
```

## Residual Rust-only Gate Blocker

`scripts/ai/check_rust_only_runtime.sh` still fails because active Rust product
paths retain PyO3/Python references outside `crates/analysis`. Current file-level
count:

```text
406 files
```

Top remaining groups:

```text
143 crates/adapters
126 crates/model
 40 crates/indicators
 24 crates/common
 13 crates/trading
  8 crates/persistence
  8 crates/network
  7 crates/execution
  7 crates/backtest
  6 crates/live
  6 crates/infrastructure
```

The recommended next cleanup slice is `crates/model`, then `crates/indicators`,
then core/common/runtime support crates, followed by adapters grouped by venue.

## Behavior Impact

No trading semantic change is intended. This removes stale binding/build
metadata and annotation residue, and moves generated C headers to Cargo
`OUT_DIR`. Rust analysis structs remain Rust structs.

## Review Status

Risk level: critical.

This task must stop at `REVIEW_REQUIRED`. Auto-merge is disabled.
