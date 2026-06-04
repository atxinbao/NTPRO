# Post-release Adapter PyO3/Cython Docs Cleanup Evidence

- Date: 2026-06-05
- Executor: Codex
- Local task name: `POST-RELEASE-ADAPTER-PYO3-CYTHON-DOCS-CLEANUP`
- Formal task file: not present in the Shrimp queue; this is a scoped
  post-release public surface cleanup.

## Goal

Clean adapter documentation where Python, PyO3, or Cython wording could be read
as a current NTPRO product entrypoint. The cleanup is scoped to Databento,
Coinbase, and Hyperliquid integration docs.

## Scope

Changed:

- `docs/integrations/databento.md`
- `docs/integrations/coinbase.md`
- `docs/integrations/hyperliquid.md`

Not changed:

- adapter implementation code;
- adapter fixtures or tests;
- CLI, dashboard, or control API behavior;
- Cargo features or workspace metadata;
- release tags or GitHub Releases;
- broad concept or developer-guide cleanup outside these adapter docs.

## Starting Inventory

Initial counts were taken from `main` with:

```bash
git show main:<file> | rg -n "PyO3|Cython|Python|nautilus_pyo3|as_legacy_cython|pyo3_price|uv pip install|pip install"
```

| File | Before | After | Classification |
|------|--------|-------|----------------|
| `docs/integrations/databento.md` | 25 | 28 | Current PyO3/Cython recommendation wording removed; retained Python loader examples are legacy context. Additional legacy labels were added so Python snippets no longer read as NTPRO product paths. |
| `docs/integrations/coinbase.md` | 11 | 3 | PyO3 import/config example removed; retained mentions explicitly say not an NTPRO entrypoint. |
| `docs/integrations/hyperliquid.md` | 11 | 8 | PyO3 client example and Python strategy example removed; retained mentions explicitly say legacy/unsupported. |

## Changes

- Coinbase:
  - removed the `nautilus_pyo3` config import example;
  - replaced "PyO3 surface available" wording with Rust adapter configuration
    wording;
  - clarified that Python/PyO3 construction paths are not NTPRO product
    entrypoints.
- Databento:
  - replaced PyO3-to-Cython conversion guidance with Rust-defined data type
    wording;
  - reclassified Python loader and `as_legacy_cython` examples as retained
    upstream legacy context;
  - removed comments that presented PyO3/Cython decoding as an optimization or
    current recommendation.
- Hyperliquid:
  - removed the PyO3 `HyperliquidHttpClient` advanced workflow example;
  - removed the Python `TradingNode` custom-data strategy example;
  - reclassified Python/PyO3 dispatch and config behavior as legacy upstream
    context;
  - kept Rust-native execution-client wording.

## Validation

| Command | Result | Notes |
|---------|--------|-------|
| `rg -n "nautilus_pyo3|pyo3_price|Convert a PyO3|Decode data to Cython objects|Decode trades to PyO3 objects|Use PyO3|PyO3 surface available|constructed from Python via|Rust and PyO3|_pyo3|PyO3-only" docs/integrations/databento.md docs/integrations/coinbase.md docs/integrations/hyperliquid.md` | passed | No matches; command exited 1 because `rg` found no current-entrypoint wording. |
| `rg -n "PyO3|Cython|Python|as_legacy_cython" docs/integrations/databento.md docs/integrations/coinbase.md docs/integrations/hyperliquid.md` | completed | 39 retained hits, all legacy/unsupported or historical Python loader context. |
| `scripts/ai/check_rust_only_runtime.sh` | passed | Reported `rust-only-runtime: ok`. |
| `scripts/ai/check_cython_removed.sh` | passed | Reported `cython-removed: ok`. |
| `scripts/ai/verify_fast.sh` | passed | Used cargo/rustc 1.95.0; workspace cargo check and clippy skipped by fast-mode defaults. |
| `bash -lc 'source scripts/ai/toolchain_env.sh && cargo check -p nautilus-cli'` | passed | Finished `dev` profile in 54.66s. A direct shell `cargo check -p nautilus-cli` first failed because Homebrew rustc 1.87.0 was on PATH; the project toolchain environment selects rustc 1.95.0 as required. |
| `git diff --check` | passed | No whitespace errors. |

## Retained References

The remaining Python/PyO3/Cython-related text in the scoped files is retained
only when it is explicitly framed as:

- legacy upstream context;
- unsupported NTPRO product path;
- historical Python loader example;
- external protocol or venue context.

## Behavior Impact

No runtime behavior changed. This is a documentation-only cleanup.

## Public API Impact

No public Rust API changed. No exported types, functions, modules, Cargo
features, crate names, binary names, or adapter APIs changed.

## Migration Note Status

No new migration note is required. This PR reinforces the existing Rust-only
migration posture by removing misleading adapter-doc language for Python,
PyO3, and Cython product paths.

## Rollback Plan

Revert this PR to restore the prior adapter documentation. No runtime, API,
dependency, or data migration is required.
