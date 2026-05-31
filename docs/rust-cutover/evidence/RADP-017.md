# RADP-017 Evidence - Add Rust Adapter Fixtures For Interactive Brokers

Date: 2026-05-31
Executor: Codex
Task ID: RADP-017
Risk: medium

## Summary

Added a compact Interactive Brokers Rust adapter scope fixture, a fixture manifest, and executable manifest tests. The manifest maps the IB instrument provider, market data, historical request, execution order, account/position, and gateway lifecycle surfaces to fixture evidence and scoped blockers for RADP-018.

This task adds fixture metadata and validation only. It does not change adapter runtime behavior, IB protocol handling, credentials, public APIs, Python/PyO3 bindings, Cython surfaces, Docker gateway behavior, or Cargo feature behavior.

## Files Changed

- `crates/adapters/interactive_brokers/test_data/rust_adapter_scope_fixture.json`
- `crates/adapters/interactive_brokers/test_data/rust_fixture_manifest.json`
- `crates/adapters/interactive_brokers/tests/fixture_manifest.rs`
- `docs/rust-cutover/evidence/RADP-017.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-017.json`

## Commands Run

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RADP-017.md
rg --files crates/adapters/interactive_brokers
python3 -m json.tool crates/adapters/interactive_brokers/test_data/rust_adapter_scope_fixture.json >/dev/null
python3 -m json.tool crates/adapters/interactive_brokers/test_data/rust_fixture_manifest.json >/dev/null
cargo fmt --check
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-interactive-brokers --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-017.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- `rust_adapter_scope_fixture.json` JSON validation: passed.
- `rust_fixture_manifest.json` JSON validation: passed.
- `cargo fmt --check`: passed.
- `cargo test -p nautilus-interactive-brokers --test fixture_manifest`: passed; 2 tests passed.
- `scripts/ai/verify_full.sh`: passed with `== verify_full complete ==`.
- Final JSON validation, agentflow role validation, and `git diff --check`: passed.

## Tests Added Or Updated

Added `crates/adapters/interactive_brokers/tests/fixture_manifest.rs`.

The test validates:

- manifest schema and task ownership;
- required fixture groups for instrument provider, market data, historical requests, execution orders, account/positions, and gateway lifecycle;
- fixture and primary source/test path existence;
- scoped blocker coverage for `IB-ADP-001` through `IB-ADP-008`;
- scope fixture gateway constraints, supported security types, and scoped rejection boundaries.

## Behavior Impact

No runtime behavior changed. The task only adds machine-checkable fixture metadata for Interactive Brokers adapter parity work.

## Public API Impact

No public API changed.

## Migration Note Status

No migration note required because this is fixture/evidence metadata and test coverage only.

## Rollback Plan

Revert this PR to remove the IB fixture manifest, scope fixture, manifest test, evidence file, and RADP-017 agentflow state changes.
