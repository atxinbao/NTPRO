# V02 Release CLI Binary Gate Evidence

Date: 2026-06-07
Executor: Codex

## Task

Local task: V02 release gate blocker cleanup.

## Goal

Keep the v0.2 release gate deterministic after the CLI crate added more than
one binary target.

## Findings

- `scripts/ai/verify_release.sh` passed `scripts/ai/verify_full.sh`.
- The release workspace build also passed.
- The next release gate step failed at the Rust CLI product surface check.
- Failure reason:
  - `cargo run -q -p nautilus-cli -- --help` could not determine which binary
    to run.
  - Cargo reported available binaries: `nautilus`, `ntpro-node`.
- This is a release script ambiguity, not a CLI compile failure.

## Changes

- `scripts/ai/verify_release.sh`
  - Uses `cargo run -q -p nautilus-cli --bin nautilus -- --help`.
  - Keeps the same help-content assertion for `backtest`, `live`, `sandbox`,
    `data`, `database`, and `blockchain`.

## Command Output Summary

- `bash -n scripts/ai/verify_release.sh`
  - Result: passed.
- `cargo run -q -p nautilus-cli --bin nautilus -- --help`
  - Result: passed.
  - Output contains the expected product-surface commands.
- `scripts/ai/verify_fast.sh`
  - Result: passed.
- `git diff --check`
  - Result: passed.

## Behavior Impact

No runtime or trading behavior changes.

The release gate now checks the intended primary CLI binary explicitly.

## Public API Impact

No public API changes.

## Migration Note

No migration note required.

## Rollback Plan

Revert the `--bin nautilus` addition in `scripts/ai/verify_release.sh` and rerun
`scripts/ai/verify_release.sh`.
