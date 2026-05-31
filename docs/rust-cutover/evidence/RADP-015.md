# RADP-015 Evidence - Close Rust Adapter Gaps For Databento Tardis

Date: 2026-05-31
Executor: Codex
Task ID: RADP-015
Risk: medium

## Summary

Added Rust adapter parity closure manifests for Databento and Tardis. Each RADP-013 gap is now recorded as closed, scoped, or deferred behind the later removal gate, with evidence paths and release-gate notes.

No adapter runtime behavior changed.

## Files Changed

- `crates/adapters/databento/test_data/rust_adapter_parity_closure.json`
- `crates/adapters/databento/tests/fixture_manifest.rs`
- `crates/adapters/tardis/test_data/rust_adapter_parity_closure.json`
- `crates/adapters/tardis/tests/fixture_manifest.rs`
- `docs/rust-cutover/evidence/RADP-015.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-015.json`

## Commands Run

```bash
python3 -m json.tool crates/adapters/databento/test_data/rust_adapter_parity_closure.json >/dev/null
python3 -m json.tool crates/adapters/tardis/test_data/rust_adapter_parity_closure.json >/dev/null
cargo fmt --check
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-databento --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-tardis --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
```

## Command Results

- Databento closure manifest JSON validation: passed.
- Tardis closure manifest JSON validation: passed.
- `cargo fmt --check`: passed.
- `cargo test -p nautilus-databento --test fixture_manifest`: passed, 2 tests passed.
- `cargo test -p nautilus-tardis --test fixture_manifest`: passed, 2 tests passed.
- `scripts/ai/verify_full.sh`: passed with `== verify_full complete ==`.

## Tests Added Or Updated

- Added Databento `rust_adapter_gap_closure_is_complete_and_scoped`.
- Added Tardis `rust_adapter_gap_closure_is_complete_and_scoped`.

Both tests require:

- the closure manifest to be valid JSON;
- every expected adapter gap to be present;
- no gap to remain open;
- every decision to include a non-empty decision and release-gate note;
- every evidence path to resolve inside the adapter crate;
- deferred Python/PyO3 cleanup gaps to require the removal gate.

## Behavior Impact

No runtime behavior, adapter behavior, public API, trading semantics, Python, PyO3, Cython, or Cargo feature behavior changed. This task only adds machine-checkable parity closure evidence and tests for that evidence.

## Public API Impact

None.

## Migration Note Status

No migration note required because there is no public API or runtime behavior change.

## Rollback Plan

Revert the closure manifest files, the related fixture manifest tests, this evidence file, and the RADP-015 agentflow state and lease updates.
