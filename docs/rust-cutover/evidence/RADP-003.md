# RADP-003 Evidence - Close Rust Adapter Gaps For Binance

Date: 2026-05-31
Executor: Codex
Task ID: RADP-003
Risk: medium

## Summary

Closed the current Binance Rust adapter parity gaps by making the supported,
scoped, deferred, and non-blocking surfaces explicit in docs and in the
executable fixture manifest.

This task does not implement new Binance runtime behavior. Instead, it turns
the RADP-001 gap inventory into a release-gate contract: Spot, USD-M Futures,
and COIN-M Futures are the current Rust factory-supported product targets;
Margin, Options, multi-product single-client registration, unsupported book
modes, generic custom data, and non-perpetual COIN-M historical open interest
remain explicitly scoped or deferred.

## Files Changed

- `crates/adapters/binance/README.md`
- `crates/adapters/binance/test_data/rust_fixture_manifest.json`
- `crates/adapters/binance/tests/fixture_manifest.rs`
- `docs/rust-cutover/inventory/binance_adapter_gaps.md`
- `docs/rust-cutover/evidence/RADP-003.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-003.json`

## Commands Run

Task setup and context:

```bash
sed -n '1,260p' docs/rust-cutover/tasks/RADP-003.md
sed -n '1,260p' docs/rust-cutover/inventory/binance_adapter_gaps.md
sed -n '1,220p' docs/rust-cutover/evidence/RADP-002.md
sed -n '1,160p' .agentflow/leases/RADP-003.json
python3 -m json.tool .agentflow/state/task_status.json
```

Targeted and required validation:

```bash
python3 -m json.tool crates/adapters/binance/test_data/rust_fixture_manifest.json >/dev/null
cargo fmt --check
cargo fmt
cargo test -p nautilus-binance --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-binance --test fixture_manifest
scripts/ai/verify_full.sh
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
```

Final local checks:

```bash
python3 -m json.tool crates/adapters/binance/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-003.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- Fixture manifest JSON validation: passed.
- First `cargo fmt --check`: failed because the updated manifest test needed
  standard rustfmt wrapping.
- `cargo fmt`: passed.
- Final `cargo fmt --check`: passed.
- Default `cargo test -p nautilus-binance --test fixture_manifest`: failed
  before compiling because the default shell `rustc` was 1.87.0 while the
  workspace requires Rust 1.95.0.
- Rust 1.95 `cargo test -p nautilus-binance --test fixture_manifest`: passed,
  1 test.
- Default `scripts/ai/verify_full.sh`: failed before validation because the
  default shell `rustc` was 1.87.0 while the workspace requires Rust 1.95.0.
- Rust 1.95 `scripts/ai/verify_full.sh`: passed and ended with
  `== verify_full complete ==`.
- Final JSON validation for the fixture manifest, task status, and lease:
  passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.

## Tests Added Or Updated

Updated `crates/adapters/binance/tests/fixture_manifest.rs`.

The test now verifies:

- the manifest was closed by `RADP-003`;
- every scoped blocker has a non-open status and a written resolution;
- every `BIN-ADP-001` through `BIN-ADP-010` gap has a closure entry;
- no closure entry remains `open`;
- every closure entry is reviewed by `RADP-003`;
- each closure entry records a decision and evidence references.

## Behavior Impact

No runtime behavior changed. No trading semantics, parser behavior, order
behavior, exchange protocol handling, credential handling, public API, Python
API, PyO3 binding, Cython surface, Cargo feature behavior, or persistence format
changed.

The practical impact is documentation and release-gate clarity: Binance Rust
adapter support is now explicitly limited to the current factory-supported Spot,
USD-M Futures, and COIN-M Futures paths unless a future task implements and
validates additional surfaces.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR does not change public API,
runtime behavior, persisted data, or user-facing configuration semantics.

## Gate Status

RADP-003 is medium risk. It closes adapter parity gaps by scope decision and
manifest validation only; it does not change adapter runtime behavior or
trading semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the Binance README update, manifest closure entries, manifest test
assertions, inventory closure-decision section, evidence file, and the
RADP-003 task state/lease updates. No runtime, persisted data, adapter protocol,
schema, Python, PyO3, Cython, or public API rollback is required.
