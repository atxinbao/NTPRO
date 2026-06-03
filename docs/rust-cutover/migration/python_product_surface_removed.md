# Python Product Surface Removed

Date: 2026-06-02
Executor: Codex
Task ID: RREM-013

Updated: 2026-06-04
Executor: Codex
Follow-up ID: RC-CLEANUP-001

## Current Status Update

This document is a historical RREM-013 evidence note with a current status
appendix. Later RREM cleanup removed the remaining PyO3/Cython/Rust crate
residue, RREL-009 made the final Rust-only release verification green, and
`ntpro-rust-only-rc.2` was created as the current tag-only release candidate.

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

RREM-013 did not remove every historical Python reference from docs, examples,
tests, or Rust comments at that time. Later cleanup removed Python examples,
Python-facing docs, and top-level Python tests from the Rust-only release
surface.

RREM-013 also did not remove every remaining PyO3 annotation or Cython
parity/build reference in Rust crates at that time. Later release gate work
handled that residue and made the final Rust-only runtime gate pass.

## Validation Notes

`scripts/ai/verify_fast.sh` passes after the removal.

At RREM-013 time, `scripts/ai/check_rust_only_runtime.sh` still failed because
active Rust source paths retained PyO3 annotations and Cython
generation/parity references. That historical blocker was resolved by later
RREM cleanup and RREL-009 release verification.
