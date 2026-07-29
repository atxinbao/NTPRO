# NTPRO Verification Guide

Date: 2026-06-04
Executor: Codex
Task ID: RHARD-007

## Purpose

This guide explains which local verification command to run for each NTPRO
work type. It keeps fast edit checks, broader regression checks, release gates,
Rust-only surface checks, Cython-removal checks, and golden trace checks
separate so users and agents do not treat every command as an every-edit
requirement.

All verification scripts pin Rust `1.95.0` through
`scripts/ai/toolchain_env.sh` before running Cargo. See
[NTPRO Rust Toolchain Verification](toolchain.md) for toolchain setup and the
Homebrew Rust PATH pitfall.

## Command Selection

| Command | Use when | Cost | What it checks | Notes |
| --- | --- | --- | --- | --- |
| `scripts/ai/verify_fast.sh` | Every PR and most local edits. | Low | Rust toolchain selection and `cargo fmt --check`. Optional Cargo check and clippy are off by default. | Required baseline for RHARD tasks. |
| `VERIFY_FAST_CARGO_CHECK=1 scripts/ai/verify_fast.sh` | A low or medium-risk change touches Rust code and a quick workspace check is appropriate. | Medium to high | Fast checks plus `cargo check --workspace --features "$NAUTILUS_RUST_FEATURES"`. | Can be slow in the full workspace. |
| `VERIFY_FAST_CLIPPY=1 scripts/ai/verify_fast.sh` | A Rust code change needs local lint evidence but not full release evidence. | Medium to high | Fast checks plus workspace clippy. | Use selectively. |
| `scripts/ai/verify_full.sh` | Runtime-facing or broad Rust changes need deeper regression evidence. | High | Fast checks, clippy, workspace tests, golden traces, and docs. | Not an every-edit command. |
| `scripts/ai/verify_release.sh` | Release candidate, final release, or owner-approved release gate work. | Very high | Full checks, release build, Rust CLI surface, Rust-only runtime check, and final Cython-removal check. | Expected to take substantially longer than fast checks. |
| `scripts/ai/check_rust_only_runtime.sh` | Confirm the active product surface remains Rust-only. | Low to medium | Rejects retained Python/PyO3/Cython product paths and verifies `nautilus-cli` exists in Cargo metadata. | Good supplemental check for release and cleanup work. |
| `scripts/ai/check_cython_removed.sh` | Confirm final Cython source/build artifacts are absent. | Low | Rejects `.pyx` / `.pxd` files and active Cython build references. | Documentation may still mention Cython as historical evidence. |
| `scripts/ai/run_golden_traces.sh` | Behavior-sensitive runtime, adapter, backtest/live, or trace work needs replay evidence. | Medium to high | Validates golden trace fixtures and runs Rust trace replay harnesses. | Use `REQUIRE_GOLDEN_REPLAY=1` for release-mode trace scope. |
| `scripts/ai/verify_cli_help.sh` | CLI product-surface changes need help-output evidence. | Medium | Runs top-level, version, and database CLI help. | First run can compile the CLI and take longer than expected. |

Related risk registers:

- [Current Ignored Tests Risk Register](../quality/ignored_tests_register.md)
  is the sole current authority for ignored Rust tests, owners, and close
  conditions.
- [Historical Ignored Tests Risk Expansion](ignored_tests_risk_register.md)
  preserves older GH-160, DRG, v0.3.1, and v0.4 classification evidence.

## Default Fast Check

Run this before opening a PR:

```bash
scripts/ai/verify_fast.sh
```

Expected output includes the pinned toolchain:

```text
== verify_fast: toolchain ==
cargo 1.95.0 ...
rustc 1.95.0 ...
== verify_fast: scope ==
fast smoke only: toolchain + cargo fmt by default
not a full workspace check, clippy gate, release gate, or golden trace gate
```

By default, `verify_fast.sh` does not run workspace Cargo check or clippy. That
keeps docs and control-plane PRs fast while still catching formatting and
toolchain mistakes.

## Full Check

Run this when a change touches broad Rust behavior, runtime paths, or shared
contracts:

```bash
scripts/ai/verify_full.sh
```

This command runs:

- `scripts/ai/verify_fast.sh`;
- workspace clippy;
- workspace tests or nextest;
- golden trace validation;
- Rust documentation generation.

Treat `verify_full.sh` as a deeper regression gate. It is not the default
command for docs-only or control-plane metadata changes.

## Release Check

Run this only for release work or when a release gate explicitly requires it:

```bash
scripts/ai/verify_release.sh
```

This command runs:

- `scripts/ai/verify_full.sh`;
- workspace release build;
- Rust CLI product-surface smoke;
- `scripts/ai/check_rust_only_runtime.sh`;
- `scripts/ai/check_cython_removed.sh`.

The release build is expected to be slow because it compiles the workspace in
release mode after the full regression path. Do not use it as a per-edit
verification command.

## Rust-Only Surface Check

Run this after cleanup, release, or public-surface changes:

```bash
scripts/ai/check_rust_only_runtime.sh
```

It verifies the final product runtime does not retain active Python, PyO3, or
Cython product paths. It also confirms `nautilus-cli` still appears in Cargo
metadata.

## Cython Removed Check

Run this when validating final removal state:

```bash
scripts/ai/check_cython_removed.sh
```

This is intentionally stricter than a runtime-only check. It rejects retained
`.pyx` and `.pxd` files and active build references. Historical migration docs
may still mention Cython.

## Golden Trace Checks

Run standard trace validation:

```bash
scripts/ai/run_golden_traces.sh
```

Run release-mode trace validation:

```bash
REQUIRE_GOLDEN_REPLAY=1 scripts/ai/run_golden_traces.sh
```

Use golden traces for behavior-sensitive changes, especially:

- order lifecycle;
- risk rejection;
- execution routing;
- position and PnL behavior;
- cache/message-bus ordering;
- backtest/live parity;
- adapter payload parsing.

## Evidence Rules

Every PR should record:

- the command that was run;
- whether it passed, failed, timed out, or was intentionally skipped;
- why a skipped command was not appropriate for the task;
- whether the result affects a final gate.

For docs-only low-risk work, `git diff --check` and `scripts/ai/verify_fast.sh`
are normally sufficient.

For medium-risk Rust product-surface work, add targeted CLI, Cargo, or example
evidence for the touched area.

For high-risk runtime, adapter, persistence, or release work, do not replace
required golden trace or release evidence with fast checks.
