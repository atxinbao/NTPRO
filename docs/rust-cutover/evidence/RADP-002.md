# RADP-002 Evidence - Add Rust Adapter Fixtures For Binance

Date: 2026-05-31
Executor: Codex
Task ID: RADP-002
Risk: medium

## Summary

Added a Rust fixture coverage manifest for the Binance adapter and a Rust test
that validates the manifest is complete and points only to existing fixture
files.

The manifest does not add new exchange payloads or change parser behavior. It
binds the existing Binance Spot and Futures fixture corpus to the Rust adapter
parity gate, including parser, user-data, execution lifecycle, SBE user-data,
market-data, and scoped blocker categories.

## Files Changed

- `crates/adapters/binance/test_data/rust_fixture_manifest.json`
- `crates/adapters/binance/tests/fixture_manifest.rs`
- `crates/adapters/binance/test_data/README.md`
- `docs/rust-cutover/evidence/RADP-002.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-002.json`

## Commands Run

Task setup and scope:

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RADP-002.md
sed -n '1,160p' .agentflow/leases/RADP-002.json
python3 -m json.tool .agentflow/state/task_status.json
```

Context:

```bash
sed -n '1,220p' crates/adapters/binance/tests/spot.rs
sed -n '1,220p' crates/adapters/binance/tests/futures.rs
sed -n '1,220p' crates/adapters/binance/test_data/README.md
sed -n '1,260p' crates/adapters/binance/test_data/SOURCES.md
rg -n "include_str!|test_data|fixture|from_str|serde_json" crates/adapters/binance/tests crates/adapters/binance/src --glob '!src/spot/sbe/generated/**'
```

Required and targeted validation:

```bash
cargo test -p nautilus-binance --test fixture_manifest
scripts/ai/verify_full.sh
cargo fmt
scripts/ai/verify_full.sh
```

Final local checks:

```bash
python3 -m json.tool crates/adapters/binance/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-002.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- `cargo test -p nautilus-binance --test fixture_manifest`: passed, 1 test.
- First `scripts/ai/verify_full.sh`: failed at the rustfmt check because one
  manifest-test assertion needed standard formatting.
- `cargo fmt`: passed and normalized the Rust test formatting.
- Second `scripts/ai/verify_full.sh`: passed and ended with
  `== verify_full complete ==`.
- JSON validation for the fixture manifest, task status, and lease: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.

## Tests Added Or Updated

Added `crates/adapters/binance/tests/fixture_manifest.rs`.

The test verifies:

- the manifest is valid JSON;
- the task ID is `RADP-002`;
- required Spot/Futures parser and lifecycle surfaces are classified;
- every fixture path listed in the manifest exists under
  `crates/adapters/binance/test_data/`;
- duplicate fixture paths are rejected;
- scoped blockers link back to `BIN-ADP-*` inventory gap IDs and are owned by
  RADP-003.

## Behavior Impact

No runtime behavior changed. No parser, data, execution, exchange protocol,
credential, public API, Python API, PyO3 binding, Cython surface, or Cargo
feature behavior changed.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR adds fixture coverage metadata and
one Rust test only.

## Gate Status

RADP-002 is medium risk. It improves adapter fixture evidence without changing
adapter runtime behavior or trading semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the fixture manifest, manifest test, README entry, evidence file, and the
RADP-002 task state/lease updates. No runtime, persisted data, adapter, schema,
Python, PyO3, Cython, or public API rollback is required.
