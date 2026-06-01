# Python Examples And Docs Removed

Date: 2026-06-02
Executor: Codex
Task ID: RREM-014

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
- Python tests and local Python tooling metadata, which are tracked as later
  cleanup or release gate residue.

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

This task does not remove Python tests under `tests/`, local helper scripts
under `scripts/`, or remaining Python/PyO3/Cython references embedded in Rust
crates and release evidence. Those are separate release cleanup surfaces and
remain visible to the Rust-only runtime gate.

Cap'n Proto schemas under `crates/serialization/schemas/capnp/` are not Python
code and are not removed by this task. RREM-015 later removes Cap'n Proto as a
separate Rust serialization feature cleanup.

## Validation Notes

The task evidence records the exact commands run and the residual blockers.
The expected remaining release blocker is the broader Rust-only runtime gate,
which still reports Python/PyO3/Cython residue outside the examples/docs scope
of RREM-014.
