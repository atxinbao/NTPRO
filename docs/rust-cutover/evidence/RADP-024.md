# RADP-024 Evidence - Close Rust Adapter Gaps For Polymarket Sandbox

Date: 2026-06-01
Executor: Codex
Task ID: RADP-024
Risk: medium

## Summary

Closed Polymarket and Sandbox adapter parity gaps by adding compact closure manifests. Each RADP-022 gap is now classified as closed, scoped, or deferred behind the Rust-only removal gate, with crate-local evidence paths and release-gate notes.

No adapter runtime behavior changed. This task only adds machine-checkable closure evidence and test assertions for existing RADP-023 fixture manifests.

## Files Changed

- `crates/adapters/polymarket/test_data/rust_adapter_parity_closure.json`
- `crates/adapters/polymarket/tests/fixture_manifest.rs`
- `crates/adapters/sandbox/test_data/rust_adapter_parity_closure.json`
- `crates/adapters/sandbox/tests/fixture_manifest.rs`
- `docs/rust-cutover/evidence/RADP-024.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-024.json`

## Commands Run

- `python3 -m json.tool crates/adapters/polymarket/test_data/rust_adapter_parity_closure.json >/dev/null`
- `python3 -m json.tool crates/adapters/sandbox/test_data/rust_adapter_parity_closure.json >/dev/null`
- `cargo fmt --check`
- `cargo fmt`
- `cargo fmt --check`
- `PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-polymarket --test fixture_manifest`
- `PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-sandbox --test fixture_manifest`
- `PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 scripts/ai/verify_full.sh`

## Command Results

- Closure manifest JSON validation: passed.
- First `cargo fmt --check` found formatting drift; `cargo fmt` then `cargo fmt --check` passed.
- Targeted Polymarket fixture tests passed: 3 tests.
- Targeted Sandbox fixture tests passed: 4 tests.
- `scripts/ai/verify_full.sh` passed with `== verify_full complete ==`.

## Tests Added Or Updated

Updated:

- `crates/adapters/polymarket/tests/fixture_manifest.rs`
- `crates/adapters/sandbox/tests/fixture_manifest.rs`

The updated tests require closure manifests to be valid JSON, cover every RADP-022 PM/SBX gap listed in RADP-023 fixture blockers, contain no `open` status, include non-empty decision, release note, and evidence paths, resolve crate-local evidence, and put deferred Python/PyO3 gaps behind the removal gate.

## Behavior Impact

No runtime behavior, adapter behavior, public API, trading semantics, Python/PyO3/Cython, or Cargo feature behavior changed. Impact is release-gate clarity only.

## Public API Impact

None.

## Migration Note Status

No migration note required because no public API or runtime behavior changed.

## Risk And Gate Status

RADP-024 is medium risk. Auto-merge is allowed after local validation and GitHub smoke pass.

## Rollback Plan

Revert closure manifests, related fixture manifest tests, this evidence file, and RADP-024 state and lease updates.
