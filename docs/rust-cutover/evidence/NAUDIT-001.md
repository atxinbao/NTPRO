# NAUDIT-001 Evidence - Python Package Metadata Cleanup

Date: 2026-06-05
Executor: Codex
Task ID: NAUDIT-001
Risk: critical
Status: REVIEW_REQUIRED
PR: https://github.com/atxinbao/NTPRO/pull/175

## 中文摘要

这次处理的是 Rust-only 对外定位和 root `pyproject.toml` 的冲突。

改动后的边界是：

- NTPRO 不再把 root `pyproject.toml` 当成 Python 包清单。
- root `pyproject.toml` 只保留本地 helper/tooling 配置，例如 `uv`
  dependency groups、ruff、mypy、pytest、coverage。
- `uv.lock` 不再把当前仓库记录成 editable Python package。
- Rust-only gate 会拦截 root `[project]`、Python package classifiers、
  Python runtime dependencies、root extras、upstream package URL 和 editable
  root lock package。

没有做的事：

- 没有恢复 Python/PyO3/Cython 产品面。
- 没有实现 CLI runtime。
- 没有修改交易语义。
- 没有创建 tag 或 GitHub Release。
- 没有自动合并；critical 任务必须停在 review。

## Metadata Decision

Root `pyproject.toml` is retained in place as local helper-tool configuration.
It is not moved to another directory in this PR because the local scripts and
pre-commit tooling still read `[tool.uv]`, `[tool.ruff]`, `[tool.mypy]`,
`[tool.pytest.*]`, and `[tool.coverage.*]` from the root.

Root Python package metadata is removed instead:

- `[project]`
- `[project.urls]`
- `[project.optional-dependencies]`
- root Python runtime `dependencies`
- `requires-python`
- Python package classifiers
- upstream package homepage/repository/docs URLs

The detailed product/tooling boundary is recorded in
`docs/rust-cutover/product/PYTHON_TOOLING_BOUNDARY.md`.

## Files Changed

- `pyproject.toml`
- `uv.lock`
- `Makefile`
- `scripts/package-version.sh`
- `scripts/test.sh`
- `scripts/test-performance.sh`
- `scripts/test-coverage.sh`
- `scripts/ai/check_rust_only_runtime.sh`
- `docs/rust-cutover/product/PYTHON_TOOLING_BOUNDARY.md`
- `docs/rust-cutover/migration/python_product_surface_removed.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/NAUDIT-001.json`

## Behavior Impact

No trading behavior changed.

Local helper tooling remains allowed, but root Python package install/build
signals are removed. Commands that only need helper dependency groups should
sync `uv` groups without package extras or root project installation.

## Public API Impact

The Python package product surface remains unsupported. This PR makes the
metadata match that public Rust-only position.

Rust crate APIs and Rust CLI command behavior are not intentionally changed.

## Migration Note Status

Updated:

- `docs/rust-cutover/migration/python_product_surface_removed.md`

Added:

- `docs/rust-cutover/product/PYTHON_TOOLING_BOUNDARY.md`

## Validation Commands

Final commands:

```bash
git diff --check
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_cython_removed.sh
scripts/ai/verify_fast.sh
python3 - <<'PY'
from pathlib import Path
import tomllib
for path in ["pyproject.toml", "uv.lock", "Cargo.toml"]:
    tomllib.loads(Path(path).read_text())
    print(f"OK {path}")
PY
scripts/check-no-build-packages.sh
bash -n scripts/package-version.sh scripts/test.sh scripts/test-performance.sh scripts/test-coverage.sh scripts/ai/check_rust_only_runtime.sh
scripts/package-version.sh
rg -n "^\[project\]|^\[project\.urls\]|^\[project\.optional-dependencies\]|Programming Language :: Python|name = \"nautilus_trader\"|name = \"nautilus-trader\"|source = \{ editable = \"\.\" \}|--all-extras|--no-install-package nautilus_trader" pyproject.toml uv.lock Makefile scripts docs/rust-cutover/product/PYTHON_TOOLING_BOUNDARY.md docs/rust-cutover/evidence/NAUDIT-001.md
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/NAUDIT-001.json
scripts/ai/validate_agentflow_roles.py
source scripts/ai/toolchain_env.sh && cargo check -p nautilus-cli
uv lock --locked
```

## Validation Results

- `git diff --check`: passed.
- `scripts/ai/check_rust_only_runtime.sh`: passed.
- `scripts/ai/check_cython_removed.sh`: passed.
- `scripts/ai/verify_fast.sh`: passed with Rust/Cargo `1.95.0`; as expected,
  this is a fast smoke and skips workspace cargo check and clippy by default.
- TOML parse for `pyproject.toml`, `uv.lock`, and `Cargo.toml`: passed.
- `scripts/check-no-build-packages.sh`: passed; `pyproject.toml` has 142
  `no-build-package` entries in sync with `uv.lock`.
- Shell syntax for changed scripts: passed.
- `scripts/package-version.sh`: passed and returns Cargo workspace version
  `0.58.0`.
- JSON parse for `.agentflow/state/task_status.json` and
  `.agentflow/leases/NAUDIT-001.json`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `cargo check -p nautilus-cli` with project toolchain: passed.
- Targeted residue scan: only matched this evidence file and
  `scripts/ai/check_rust_only_runtime.sh` itself. There are no scoped matches
  in `pyproject.toml`, `uv.lock`, `Makefile`, or helper scripts.
- `uv lock --locked`: not usable in this local shell because local `uv` is
  `0.11.12` while `pyproject.toml` requires `0.11.14`. Lock consistency was
  therefore validated with TOML parsing and `scripts/check-no-build-packages.sh`.

## Review Status

PR #175 is open and ready for human / gatekeeper review. Auto-merge is not
enabled because NAUDIT-001 is a critical-risk task.

## Known Residuals

Legacy Python wheel workflows and release helper scripts still exist under
`.github/**` and `scripts/ci/**`. They are historical publication surfaces and
should be handled by a separate CI/release workflow cleanup task rather than
folded into NAUDIT-001.

## Rollback Plan

Revert this PR to restore the previous root Python package metadata, editable
root package lock entry, and helper command wiring.
