# Python Product Surface Removed

Date: 2026-06-02
Executor: Codex
Task ID: RREM-013

## Summary

RREM-013 removes the Python package product surface from the Rust-only cutover
workspace.

Removed product paths:

- `python/`
- `nautilus_trader/`
- `build.py`

The root `pyproject.toml` remains as local Python tooling metadata for hooks
and helper scripts, but it no longer declares the removed Poetry build backend,
wheel include paths, or `build.py` build script.

## User Impact

Python package imports are no longer a supported product path in this
workspace. Code that imports modules such as `nautilus_trader.backtest`,
`nautilus_trader.live`, `nautilus_trader.model`, or
`python.nautilus_trader` must move to the Rust product surface.

Python wheel build commands are also retired. The Makefile no longer drives the
removed `build.py` package build, and the v2 Python wheel workflow no longer
builds or publishes Python wheels.

## Replacement Path

Use the Rust workspace and CLI/product entries:

- `cargo metadata --format-version=1`
- `scripts/ai/verify_fast.sh`
- `cargo build --workspace`
- `cargo test -p <crate>`
- Rust examples and Rust cutover docs under `docs/rust-cutover/`

## Not Removed In This Task

RREM-013 does not remove every historical Python reference from docs, examples,
tests, or Rust comments. Those references are now migration or release-cleanup
residuals unless they are active product package paths.

This task also does not remove every remaining PyO3 annotation or Cython
parity/build reference in Rust crates. Those residuals still block the final
Rust-only runtime gate and must be handled by follow-up release gate work.

## Validation Notes

`scripts/ai/verify_fast.sh` passes after the removal.

`scripts/ai/check_rust_only_runtime.sh` still fails because active Rust source
paths retain PyO3 annotations and Cython generation/parity references. This is
recorded as a release blocker instead of being treated as complete.
