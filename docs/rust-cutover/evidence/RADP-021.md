# RADP-021 Evidence - Close Rust Adapter Gaps For Betfair Architect AX

Date: 2026-05-31
Executor: Codex
Task ID: RADP-021
Risk: medium

## Summary

Closed the Betfair and Architect AX adapter parity gaps by recording compact
parity closure manifests. Each RADP-019 gap is now classified as closed,
scoped, or deferred behind the later Rust-only removal gate, with crate-local
evidence paths and release-gate notes.

No adapter runtime behavior changed. This task only adds machine-checkable
closure evidence and test assertions for the existing Betfair and Architect AX
fixture manifests.

## Files Changed

- `crates/adapters/betfair/test_data/rust_adapter_parity_closure.json`
- `crates/adapters/betfair/tests/fixture_manifest.rs`
- `crates/adapters/architect_ax/test_data/rust_adapter_parity_closure.json`
- `crates/adapters/architect_ax/tests/fixture_manifest.rs`
- `docs/rust-cutover/evidence/RADP-021.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-021.json`

## Commands Run

```bash
sed -n '1,260p' docs/rust-cutover/tasks/RADP-021.md
sed -n '1,260p' docs/rust-cutover/evidence/RADP-019.md
python3 -m json.tool crates/adapters/betfair/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool crates/adapters/architect_ax/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool crates/adapters/betfair/test_data/rust_adapter_parity_closure.json >/dev/null
python3 -m json.tool crates/adapters/architect_ax/test_data/rust_adapter_parity_closure.json >/dev/null
cargo fmt --check
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-betfair --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-architect-ax --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-021.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- Betfair fixture and closure manifest JSON validation: passed.
- Architect AX fixture and closure manifest JSON validation: passed.
- `cargo fmt --check`: initial run found formatting drift in the Architect AX
  fixture manifest test; `cargo fmt && cargo fmt --check` passed after
  formatting.
- `cargo test -p nautilus-betfair --test fixture_manifest`: passed, 3 tests.
- `cargo test -p nautilus-architect-ax --test fixture_manifest`: passed, 3 tests.
- `scripts/ai/verify_full.sh`: passed with `== verify_full complete ==`.
- Final JSON validation, agentflow role validation, and `git diff --check`:
  passed.

## Tests Added Or Updated

Updated:

- `crates/adapters/betfair/tests/fixture_manifest.rs`
- `crates/adapters/architect_ax/tests/fixture_manifest.rs`

The new closure checks require:

- `rust_adapter_parity_closure.json` to exist and be valid JSON;
- every RADP-019 Betfair and Architect AX gap to have a RADP-021 closure entry;
- no closure entry to remain `open`;
- every decision to include a non-empty decision, release-gate note, and
  resolvable crate-local evidence path;
- deferred Python/PyO3 cleanup gaps to require the later removal gate.

## Behavior Impact

No runtime behavior, adapter behavior, public API, trading semantics, Python,
PyO3, Cython, or Cargo feature behavior changed. The practical impact is
release-gate clarity: Betfair and Architect AX adapter parity is now represented
as supported-with-constraints or deferred removal-gate work.

## Public API Impact

None.

## Migration Note Status

No migration note required because this PR documents adapter parity closure
without changing public API or runtime behavior.

## Risk And Gate Status

RADP-021 is medium risk. It closes adapter parity gaps by scope decision and
machine-checkable evidence only. Auto-merge is allowed after local validation
and GitHub smoke pass.

## Rollback Plan

Revert the closure manifest files, the related fixture manifest tests, this
evidence file, and the RADP-021 agentflow state and lease updates.
