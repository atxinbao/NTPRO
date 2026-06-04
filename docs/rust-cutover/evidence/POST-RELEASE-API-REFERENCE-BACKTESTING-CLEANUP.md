# Post-release API Reference and Backtesting Cleanup Evidence

- Date: 2026-06-05
- Executor: Codex
- Local task name: `POST-RELEASE-API-REFERENCE-BACKTESTING-CLEANUP`
- Formal task file: not present in the Shrimp queue; this is a scoped
  post-release public surface cleanup prompted by audit findings.

## Goal

Fix the highest-priority public documentation conflict reported in the audit:
the retained Python API reference and the backtesting concept guide could still
be read as current NTPRO product entrypoints.

## Scope

Changed:

- `docs/api_reference/**`
- `docs/concepts/backtesting.md`
- `docs/developer_guide/docs.md`

Not changed:

- Rust source code;
- CLI runtime wiring;
- adapter behavior;
- trading semantics;
- Cargo features or workspace metadata;
- Python, PyO3, Cython, `build.py`, or package files;
- release tags or GitHub Releases.

## Changes

- Renamed `docs/api_reference/index.md` from `Python API` to
  `Legacy Upstream Python API Appendix`.
- Added a legacy warning to every Markdown page under `docs/api_reference/**`.
- Kept the retained `nautilus_trader.*` `automodule` pages as lineage and
  migration context, not current product API evidence.
- Updated `docs/developer_guide/docs.md` so reference docs point at Rust crate
  docs and product contracts, while `api_reference/` is explicitly legacy.
- Updated `docs/concepts/backtesting.md` so the page starts from the Rust-only
  product path and points current runnable users to
  `docs/how_to/run_rust_backtest.md`.
- Reclassified Python `BacktestEngine` / `BacktestNode` snippets in
  `docs/concepts/backtesting.md` as retained upstream examples for historical
  behavior and matching-engine semantics.

## Classification

| Item | Count | Classification |
|------|-------|----------------|
| `docs/api_reference/**/*.md` and `docs/api_reference/*.md` files | 38 | All now include a legacy upstream Python API warning. |
| Retained `automodule:: nautilus_trader` references | 218 | Kept only inside the legacy appendix. |
| Current-path stale phrases checked in API/backtesting docs | 0 | No remaining `# Python API`, `Python script`, old latest/nightly branch text, or Sphinx version-selector text. |

## Validation

| Command | Result | Notes |
|---------|--------|-------|
| `find docs/api_reference -type f -name '*.md' \| wc -l` and `rg -l "This page is a legacy upstream Python API appendix" docs/api_reference \| wc -l` | passed | Both counts are 38. |
| `rg -c "automodule:: nautilus_trader" docs/api_reference` aggregate | completed | 218 retained automodule references, all inside the legacy appendix. |
| `rg -n "^# Python API|Python script|Auto-generated from the latest source|develop branch|nightly branch|Select the version" docs/api_reference docs/concepts/backtesting.md docs/developer_guide/docs.md` | passed | No matches; command exited 1 because `rg` found no stale current-path phrases. |
| `git diff --check` | passed | No whitespace errors before this evidence file was created. |
| `scripts/ai/check_rust_only_runtime.sh` | passed | Reported `rust-only-runtime: ok`. |
| `scripts/ai/check_cython_removed.sh` | passed | Reported `cython-removed: ok`. |
| `scripts/ai/verify_fast.sh` | passed | Used cargo/rustc 1.95.0; workspace cargo check and clippy skipped by fast-mode defaults. |
| `bash -lc 'source scripts/ai/toolchain_env.sh && cargo check -p nautilus-cli'` | passed | Finished `dev` profile in 0.65s. |

## Behavior Impact

No runtime behavior changed. This is a documentation-only cleanup.

## Public API Impact

No public Rust API changed. No exported types, functions, modules, Cargo
features, crate names, binary names, or adapter APIs changed.

## Migration Note Status

No new migration note is required. This PR clarifies that retained Python API
reference pages are historical appendix material, not the NTPRO Rust-only API.

## Rollback Plan

Revert this PR to restore the prior documentation. No runtime, dependency, API,
data, or release rollback is required.
