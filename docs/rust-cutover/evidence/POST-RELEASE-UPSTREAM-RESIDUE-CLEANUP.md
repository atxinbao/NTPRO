# Post-release Upstream Residue Cleanup Evidence

- Date: 2026-06-05
- Executor: Codex
- Local task name: `POST-RELEASE-UPSTREAM-RESIDUE-CLEANUP`
- Formal task file: not present in the Shrimp queue; this is a scoped
  post-release public surface cleanup.

## Goal

Clean the remaining public-facing upstream NautilusTrader source/documentation
links and Python package install commands that could make users think NTPRO is
still using upstream `develop`, `nautilustrader.io`, or Python package entrypoints
as the current product path.

## Starting Inventory

The first inventory excluded `docs/rust-cutover/**` because those files are
migration, task, release, and evidence records where historical references are
expected.

| Category | Count | Classification |
|----------|-------|----------------|
| `nautilustrader.io` / `nautechsystems/nautilus_trader` lines | 54 lines in 33 files | Mixed: public link residue plus historical issue/commit references. |
| Replaceable upstream `tree/develop` or `blob/develop` links | 39 lines | Cleaned in this PR. |
| Upstream issue / PR / commit references | 13 lines | Retained as historical or regression context. |
| `nautilustrader.io` links | 1 line | Cleaned in this PR. |
| Python package install entrypoints using `nautilus_trader[...]` | 4 lines | Cleaned or reworded in this PR. |

## Scope

Changed:

- public tutorial links under `docs/tutorials/**`;
- public integration links under `docs/integrations/**`;
- selected concept links under `docs/concepts/**`;
- selected developer-guide setup wording;
- one docs.rs-facing source-doc link in `crates/plugin/src/bridge/mod.rs`.

Not changed:

- upstream issue, PR, or commit references used as historical bug/regression
  context;
- `docs/rust-cutover/**` migration, inventory, task, release, and evidence
  records;
- runtime behavior;
- adapter behavior;
- CLI, dashboard, or control API behavior;
- Cargo feature behavior;
- release tags or GitHub Releases.

## Changes

- Retargeted upstream `tree/develop` and `blob/develop` source links to
  `https://github.com/atxinbao/NTPRO/.../main/...`.
- Retargeted integration contributing-guide links to the NTPRO repository.
- Replaced the Tardis `nautilustrader.io` venues link with an in-document
  reference to the local Venues section.
- Replaced upstream clone instructions with the NTPRO `main` repository path.
- Replaced Python `nautilus_trader[...]` install commands in Betfair, IB,
  Polymarket, reports, and visualization docs with Rust-only wording or Cargo
  validation examples.
- Replaced a concept link to the removed Cython `margin.pyx` file with the Rust
  instrument model path.

## Validation

| Command | Result | Notes |
|---------|--------|-------|
| Targeted `rg` for upstream `tree/blob/develop`, `nautilustrader.io`, and `nautilus_trader[...]` install commands outside `docs/rust-cutover/**` | passed | No matches; command exited 1 because `rg` found no lines. |
| Targeted `rg` for upstream issue / PR / commit links outside `docs/rust-cutover/**` | completed | 13 retained historical references, listed below. |
| Link target existence spot-check for NTPRO `tree/main` and `blob/main` URLs | passed | Every extracted target path exists locally. |
| `scripts/ai/check_rust_only_runtime.sh` | passed | Reported `rust-only-runtime: ok`. |
| `scripts/ai/check_cython_removed.sh` | passed | Reported `cython-removed: ok`. |
| `scripts/ai/verify_fast.sh` | passed | Used cargo/rustc 1.95.0; workspace cargo check and clippy skipped by fast-mode defaults. |
| `git diff --check` | passed | No whitespace errors. |

## Retained Historical References

The remaining `nautechsystems/nautilus_trader` references outside
`docs/rust-cutover/**` are all issue, PR, or commit links. They are retained
because they document historical bugs, regression provenance, or upstream
implementation history instead of directing users to an upstream product
entrypoint.

- `docs/developer_guide/environment_setup.md`: upstream PR reference for Rust
  analyzer context.
- `docs/concepts/dst.md`: upstream issue reference.
- `docs/concepts/data.md`: upstream commit reference.
- `docs/concepts/logging.md`: upstream issue reference.
- `docs/how_to/configure_live_trading.md`: upstream issue reference.
- `docs/integrations/hyperliquid.md`: upstream issue references.
- `crates/portfolio/src/manager.rs`: regression test issue reference.
- `crates/persistence/tests/test_feather.rs`: regression test issue reference.
- `crates/adapters/sandbox/tests/execution.rs`: regression test issue reference.
- `crates/adapters/deribit/src/websocket/parse.rs`: regression test issue
  references.

## Behavior Impact

No runtime behavior changed. This is a documentation and source-doc link cleanup
only.

## Public API Impact

No public Rust API changed. No exported types, functions, modules, Cargo
features, crate names, binary names, or adapter APIs changed.

## Migration Note Status

No new migration note is required. This PR reinforces the existing Rust-only
migration posture by removing misleading upstream and Python package product
entrypoints from public docs.

## Rollback Plan

Revert this PR to restore the old upstream links and Python package install
wording. No runtime, API, dependency, or data migration is required.
