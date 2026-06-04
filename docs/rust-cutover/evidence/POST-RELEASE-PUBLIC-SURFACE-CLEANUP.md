# Post-release Public Surface Cleanup Evidence

- Date: 2026-06-05
- Executor: Codex
- Local task name: `POST-RELEASE-PUBLIC-SURFACE-CLEANUP`
- Formal task file: not present in the Shrimp queue; this is a post-release public
  surface cleanup requested in the current Codex thread.

## Goal

Clean the first public layer after the Rust-only release so readers do not see
NTPRO as an upstream NautilusTrader Python/PyO3/Cython product.

This PR is intentionally scoped to public documentation entry points, workspace
metadata, and crate README identity cleanup. It does not implement CLI runtime
wiring, dashboard UI, control API endpoints, observability DTOs, live connection
behavior, or trading semantic changes.

## Files Changed

- `Cargo.toml`
  - changed workspace `documentation`, `repository`, and `homepage` metadata to
    the NTPRO repository.
- `docs/concepts/overview.md`
  - rewrote the entry page as NTPRO Rust-only scope and removed Python control
    plane/PyO3/Cython product-path wording.
- `docs/concepts/architecture.md`
  - removed the upstream architecture image link and replaced Python/Cython code
    structure guidance with NTPRO Rust-only structure.
- `docs/concepts/index.md`
  - added Rust-only note and removed Python/PyO3 wording from key index summaries.
- `docs/concepts/reports.md`
  - replaced the upstream Python/pandas report tutorial with an NTPRO Rust-only
    status page and legacy-path warning.
- `docs/concepts/rust.md`
  - removed upstream `develop` dependency examples and v1 Cython / v2 PyO3
    implementation-choice guidance; kept Rust workspace path guidance.
- `docs/developer_guide/index.md`
  - rewrote the developer guide entry as Rust-only and labeled legacy Python notes.
- `docs/developer_guide/testing.md`
- `docs/developer_guide/spec_data_testing.md`
- `docs/developer_guide/spec_exec_testing.md`
  - added warnings that Python/PyO3/Cython testing content is legacy context,
    while Rust/Cargo and `scripts/ai/` evidence is current.
- `crates/**/README.md`
  - removed or replaced upstream build badges, repository links, homepage links,
    Discord links, logo image links, and high-precision documentation links.
  - retained crate names, binary names, and NautilusTrader license lineage.

## Validation

| Command | Result | Notes |
|---------|--------|-------|
| `scripts/ai/check_rust_only_runtime.sh` | passed | Reported `rust-only-runtime: ok`. |
| `scripts/ai/check_cython_removed.sh` | passed | Reported `cython-removed: ok`. |
| `scripts/ai/verify_fast.sh` | passed | Used cargo/rustc 1.95.0. Workspace cargo check and clippy were skipped by fast-mode defaults. |
| `cargo check -p nautilus-cli` | failed first run | Failed with system rustc 1.87.0, below workspace requirement 1.95.0. This was an environment/toolchain issue, not a code error. |
| `source scripts/ai/toolchain_env.sh && cargo check -p nautilus-cli` | passed | Finished dev profile in 10m 07s using the project 1.95.0 toolchain. |
| `rg -n "Python control plane|PyO3|Cython|uv pip install|nautechsystems/nautilus_trader|nautilustrader.io" README.md docs crates Cargo.toml` | completed | 1339 lines after this evidence file was added. Matches are classified below instead of requiring zero hits. |

Final pattern count summary:

| Pattern | Files | Matches | Classification |
|---------|-------|---------|----------------|
| `Python control plane` | 1 | 2 | evidence text only; product wording cleared from public entry pages |
| `PyO3` | 305 | 911 | mostly migration/history/legacy warnings plus old deep docs |
| `Cython` | 293 | 803 | mostly migration/history/legacy warnings plus old deep docs |
| `uv pip install` | 8 | 11 | unsupported/migration notes plus old integration docs outside this PR |
| `nautechsystems/nautilus_trader` | 37 | 61 | old tutorial/integration/source-code links outside this PR plus this evidence classification |
| `nautilustrader.io` | 42 | 99 | source module docs and old deep docs outside this PR plus this evidence classification |

## `rg` Classification

### Cleaned

- `Cargo.toml` no longer points workspace metadata at upstream
  `nautechsystems/nautilus_trader` or `nautilustrader.io`.
- `crates/**/README.md` no longer contains upstream repository/homepage/logo/Discord
  identity links.
- `docs/concepts/overview.md` no longer presents Python as the current control
  plane or PyO3/Cython as current product paths.
- `docs/concepts/architecture.md` no longer embeds the upstream architecture image
  or presents Python/Cython package structure as the NTPRO product structure.
- `docs/concepts/reports.md` no longer instructs users to install
  `nautilus_trader[visualization]`; it only mentions that command as unsupported
  legacy context.
- `docs/concepts/rust.md` no longer tells users to depend on the upstream
  NautilusTrader `develop` branch.

### Reasonably Retained: unsupported / migration / historical

- `README.md` and `docs/getting_started/*` retain Python/PyO3/Cython terms only
  to say those surfaces are unsupported.
- `docs/rust-cutover/**` retains many Python/PyO3/Cython terms because those files
  are migration records, release evidence, task definitions, and historical gate
  decisions.
- Updated concept/developer entry pages retain Python/PyO3/Cython terms in warning
  blocks that explicitly mark them as legacy or unsupported.
- Issue links in Rust tests or source comments may still point at upstream issues
  as regression provenance.

### Follow-up Cleanup: outside this PR scope

- `crates/**/src/lib.rs` still contains crate-level docs with upstream
  `NautilusTrader` links and high-precision docs links. This should be a separate
  crate-doc cleanup slice because it touches public docs.rs-style module docs, not
  README metadata.
- Deep concept pages such as custom data, greeks, instruments, logging, data, order
  book, accounting, and continuous futures still contain legacy Python/PyO3/Cython
  explanations. They should be handled as topic-by-topic legacy cleanup, not in this
  entry-page PR.
- Integration docs under `docs/integrations/**` still include upstream links,
  Python install examples, and PyO3/Cython details. They need an adapter-specific
  public docs cleanup pass so adapter support status remains accurate.
- How-to docs still contain upstream example links. They should be moved to local
  repo paths or current NTPRO examples in a separate how-to cleanup.

## Behavior Impact

No trading behavior changed. The edits are documentation and metadata only.

## Public API Impact

No Rust public API changed. No crate names, binary names, function signatures,
Cargo features, or runtime behavior were changed.

## Migration Note Status

Migration note is included in the updated public docs: Python, PyO3, Cython,
wheel, PyPI, and upstream Python package paths are unsupported NTPRO product
surfaces. Remaining occurrences must be interpreted as unsupported, legacy,
migration, or historical context unless a future task adds Rust implementation
and release evidence.

## Rollback Plan

Revert this PR to restore the previous public documentation and metadata wording.
Because there are no runtime or Cargo dependency graph changes, rollback should
not require data migration, API migration, or build-system changes.
