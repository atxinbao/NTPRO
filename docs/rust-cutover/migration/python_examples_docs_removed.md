# Python Examples And Docs Removed

Date: 2026-06-02
Executor: Codex
Task ID: RREM-014

Updated: 2026-06-04
Executor: Codex
Follow-up ID: RC-CLEANUP-001

## Current Status Update

This document is a historical RREM-014 evidence note with a current status
appendix. Later cleanup removed the top-level legacy Python tests under
`tests/**/*.py`, and RREL-009 made the final Rust-only release verification
green.

## Summary

RREM-014 removes the legacy Python examples and Python documentation code
surfaces from the Rust-only cutover workspace.

Removed surfaces:

- non-Rust example trees under `examples/`;
- Python documentation snippets stored as runnable `.py` files under `docs/`;
- legacy tutorial panel renderer scripts under `docs/tutorials/assets/**`;
- image assets that only belonged to deleted Python tutorial pages.

Retained surfaces:

- Rust examples under `examples/rust/`;
- Rust cutover docs under `docs/rust-cutover/`;
- local Python helper scripts and tooling metadata, which are repository
  automation only and not product surfaces.

## User Impact

Users should no longer start from `examples/backtest`, `examples/live`,
`examples/sandbox`, `examples/other`, or `examples/utils`. Those directories
were Python-first product examples and are removed from this Rust-only
workspace.

Documentation no longer points users to removed Python example scripts as the
supported path. Where a Rust replacement already exists, docs now point to the
Rust examples and Rust cutover guidance. Where a direct Rust replacement is
not available, docs call out the legacy removal instead of silently keeping a
broken Python link.

## Replacement Path

Use the Rust product surface:

- `examples/rust/README.md`
- `examples/rust/backtest/README.md`
- `docs/getting_started/index.md`
- `docs/how_to/index.md`
- `docs/tutorials/index.md`
- `docs/rust-cutover/migration/rust_only_migration_guide.md`

Useful local verification commands:

- `cargo metadata --format-version=1`
- `cargo fmt --check`
- `scripts/ai/verify_fast.sh`

## Not Removed In This Task

This task did not remove Python tests under `tests/` at RREM-014 time. Later
RC cleanup removed the tracked top-level Python tests under `tests/**/*.py`.
Local helper scripts under `scripts/` remain as repository automation.

Cap'n Proto schemas under `crates/serialization/schemas/capnp/` are not Python
code and are not removed by this task. RREM-015 later removes Cap'n Proto as a
separate Rust serialization feature cleanup.

## Validation Notes

The task evidence records the exact commands run at RREM-014 time. Later
release work made the broader Rust-only release gate pass in RREL-009.
