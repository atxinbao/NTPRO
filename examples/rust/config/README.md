# Rust Config Validation Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

Updated: 2026-07-15
Executor: Codex
Task ID: DEXG-002

This directory is reserved for shared Rust config validation examples driven by
the `nautilus config` CLI.

## Command Contract

```bash
cargo run -q -p nautilus-cli -- config validate --kind backtest --config examples/rust/backtest/minimal_dry_run.toml
cargo run -q -p nautilus-cli -- config validate --kind sandbox --config examples/rust/sandbox/sandbox_smoke.toml
cargo run -q -p nautilus-cli -- config validate --kind live --config examples/rust/live/live_init_smoke.toml
cargo run -q -p nautilus-cli -- config validate --kind data --config examples/rust/data/catalog_audit.toml
```

## Current Status

`config validate` is runnable for the documented backtest, sandbox, live, and
data config kinds. It dispatches to the same scoped Rust validators used by the
owning workflow commands and may write an optional owner-visible validation
artifact.

Validation does not run a workflow, connect to an external venue, submit an
order, or authorize production operation. The accepted config shape remains
bounded by each workflow's Rust validator.

## Validation Boundary

- The selected `--kind` maps to a Rust config model.
- Validation reports failing section and field details when known.
- The command does not import Python, require PyO3, or require Cython artifacts.
