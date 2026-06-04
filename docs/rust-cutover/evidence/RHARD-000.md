# RHARD-000 Public Release Surface Cleanup Evidence

Date: 2026-06-04
Executor: Codex
Task ID: RHARD-000
Risk: low

## Scope

RHARD-000 cleans the public documentation surface after the formal
`ntpro-rust-only-v0.1.0` release.

Allowed work:

- update README release wording;
- replace Python/PyPI installation guidance with the Rust-only source/Cargo
  path;
- remove Docker/Jupyter as a default getting-started path;
- mark remaining Python-heavy tutorials as legacy migration references;
- record local public-surface audit evidence.

No Rust runtime, adapter, Cargo workspace, CI, release tag, or GitHub Release
behavior was changed.

## Changes

- `README.md` now describes NTPRO as a formal Rust-only release workspace, not
  a release-candidate workspace.
- `docs/getting_started/installation.md` was rewritten around the supported
  Rust-only source workflow:
  - clone `ntpro-rust-only-v0.1.0`;
  - install Rust `1.95.0`;
  - run `nautilus-cli` help commands through Cargo;
  - use local verification scripts.
- `docs/getting_started/index.md` no longer presents Jupyter Docker images as
  the starting path.
- `docs/tutorials/index.md` points new users at NTPRO Rust docs instead of
  upstream latest/nightly docs.
- Python-heavy tutorial pages now carry explicit legacy migration warnings and
  no longer list PyPI/NautilusTrader package installation as a prerequisite.
- `docs/rust-cutover/CONTRACT.md`,
  `docs/rust-cutover/migration/rust_only_migration_guide.md`, and
  `docs/rust-cutover/release/scope_decision_review.md` were aligned with the
  published formal release state.

## Public Surface Audit

Command:

```bash
rg -n "Python 3|PyPI|pip install|uv pip|wheel|release-candidate|release candidate" README.md docs/getting_started docs/tutorials
```

Result:

- No `release-candidate` or `release candidate` wording remains in README,
  getting-started docs, or tutorials.
- No Python/PyPI install command remains as a supported setup step.
- Remaining `PyPI`, `pip install`, `uv pip`, and `wheel` matches are only in
  explicit unsupported-path lists.

The broader tutorial audit still finds historical `nautilus_trader` imports in
legacy migration snippets. Those pages are now explicitly marked as migration
references, not supported NTPRO product entrypoints.

## Validation

```bash
git diff --check
```

Result: passed.

```bash
scripts/ai/verify_fast.sh
```

Result: passed.

Output summary:

```text
== verify_fast: toolchain ==
== verify_fast: rust fmt ==
== verify_fast: cargo check skipped; set VERIFY_FAST_CARGO_CHECK=1 to run the legacy mixed-workspace check ==
== verify_fast: clippy skipped; set VERIFY_FAST_CLIPPY=1 to run it in fast mode ==
== verify_fast complete ==
```

## Behavior Impact

No trading behavior changed. This is documentation-only public release surface
cleanup.

## Review Status

Ready for PR review.
