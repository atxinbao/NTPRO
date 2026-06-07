# V02 Release Docs Serial Gate Evidence

Date: 2026-06-07
Executor: Codex

## Task

Local task: V02 release gate blocker cleanup.

## Goal

Make the release verification doc-generation step deterministic enough for the
v0.2 release gate. The previous `cargo doc --workspace --features
"arrow,ffi,high-precision,streaming,defi" --no-deps` stage reached Rust docs
after all major clippy, test, and golden trace checks passed, then stalled with
multiple concurrent `rustdoc` / `rustc` child processes sleeping at 0% CPU for
several minutes.

## Changes

- `scripts/ai/verify_full.sh`
  - Adds `VERIFY_FULL_CARGO_DOC_JOBS`, defaulting to `1`.
  - Runs `cargo doc` with `--jobs "$VERIFY_FULL_CARGO_DOC_JOBS"`.
  - Prints the selected docs job count before running the docs gate.

## Command Output Summary

- `cargo doc --workspace --features "arrow,ffi,high-precision,streaming,defi" --no-deps --jobs 1`
  - Result: passed.
  - Duration: 19m 15s.
  - Output ended with generated workspace documentation under
    `target/doc/`.

## Behavior Impact

No runtime or trading behavior changes.

## Public API Impact

No public API changes. The change only affects local release verification
orchestration.

## Migration Note

No migration note required. Operators can override the default docs parallelism
with `VERIFY_FULL_CARGO_DOC_JOBS=<n>` when the local environment can run
parallel rustdoc reliably.

## Rollback Plan

Revert the `scripts/ai/verify_full.sh` change and rerun `scripts/ai/verify_release.sh`.
