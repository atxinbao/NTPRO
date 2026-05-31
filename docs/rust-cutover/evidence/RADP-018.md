# RADP-018 Evidence - Close Rust Adapter Gaps For Interactive Brokers

Date: 2026-05-31
Executor: Codex
Task ID: RADP-018
Risk: medium

## Summary

Closed the Interactive Brokers adapter parity gaps by recording a compact parity closure manifest and making the fixture manifest tests reject open IB gaps. Each RADP-016 gap is now classified as closed, scoped, or deferred behind the Rust-only removal gate.

No adapter runtime behavior changed. This task records release-gate decisions for existing IB Rust surfaces and keeps live IB/TWS/Gateway behavior behind explicit gateway, UTC, credential, Docker, or env-gated smoke-test boundaries.

## Files Changed

- `crates/adapters/interactive_brokers/test_data/rust_adapter_parity_closure.json`
- `crates/adapters/interactive_brokers/test_data/rust_fixture_manifest.json`
- `crates/adapters/interactive_brokers/tests/fixture_manifest.rs`
- `docs/rust-cutover/evidence/RADP-018.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-018.json`

## Commands Run

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RADP-018.md
sed -n '1,220p' docs/rust-cutover/inventory/interactive_brokers_adapter_gaps.md
python3 -m json.tool crates/adapters/interactive_brokers/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool crates/adapters/interactive_brokers/test_data/rust_adapter_scope_fixture.json >/dev/null
python3 -m json.tool crates/adapters/interactive_brokers/test_data/rust_adapter_parity_closure.json >/dev/null
cargo fmt --check
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-interactive-brokers --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-018.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- Interactive Brokers fixture manifest JSON validation: passed.
- Interactive Brokers scope fixture JSON validation: passed.
- Interactive Brokers parity closure JSON validation: passed.
- `cargo fmt --check`: passed.
- `cargo test -p nautilus-interactive-brokers --test fixture_manifest`: passed; 3 tests passed.
- `scripts/ai/verify_full.sh`: passed with `== verify_full complete ==`.
- Final JSON validation, agentflow role validation, and `git diff --check`: passed.

## Tests Added Or Updated

Updated `crates/adapters/interactive_brokers/tests/fixture_manifest.rs`.

The tests now require:

- no RADP-018-owned Interactive Brokers blocker remains `open`;
- every blocker has a non-empty RADP-018 resolution;
- `rust_adapter_parity_closure.json` exists and resolves all `IB-ADP-001` through `IB-ADP-008`;
- every closure decision has a non-open status, decision text, release-gate note, and resolvable crate-local evidence path;
- deferred Python/PyO3 cleanup requires the later Rust-only removal gate.

## Behavior Impact

No runtime behavior, adapter behavior, public API, trading semantics, Python, PyO3, Cython, or Cargo feature behavior changed. This task only adds machine-checkable parity closure evidence and tests for that evidence.

## Public API Impact

None.

## Migration Note Status

No migration note required because there is no public API or runtime behavior change.

## Rollback Plan

Revert the closure manifest file, the fixture manifest resolution updates, the related fixture manifest test assertions, this evidence file, and the RADP-018 agentflow state and lease updates.
