# RADP-013 Evidence - Inventory Rust Adapter Gaps For Databento Tardis

Date: 2026-05-31
Executor: Codex
Task ID: RADP-013
Risk: medium

## Summary

Created the Databento and Tardis Rust adapter gap inventory for the R5 adapter
parity sequence. The inventory records current Rust surfaces, fixture/test
coverage, support classifications, and release-gate gaps for RADP-014 and
RADP-015.

The task is documentation and control-state only. It does not change adapter
runtime behavior, market-data decoding, exchange/data-provider protocol
handling, credential handling, public APIs, Python/PyO3 bindings, Cython
surfaces, or Cargo feature behavior.

## Files Changed

- `docs/rust-cutover/inventory/databento_tardis_adapter_gaps.md`
- `docs/rust-cutover/evidence/RADP-013.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-013.json`

## Commands Run

Context and inventory:

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RADP-013.md
sed -n '1,220p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,220p' docs/rust-cutover/CONTRACT.md
sed -n '1,220p' docs/rust-cutover/DEFINITION_OF_DONE.md
sed -n '1,240p' docs/rust-cutover/AGENT_ROLES.md
sed -n '1,260p' docs/rust-cutover/inventory/deribit_dydx_hyperliquid_adapter_gaps.md
find crates/adapters/databento crates/adapters/tardis -maxdepth 3 -type f | sort
rg -n "unsupported|Unsupported|TODO|todo|Python|pyo3|feature = \"python\"|cfg\\(feature = \"python\"\\)|subscribe_|request_|book_snapshot|book_change|trade_bar|derivative_ticker" crates/adapters/databento/src crates/adapters/tardis/src crates/adapters/databento/tests crates/adapters/tardis/tests crates/adapters/databento/README.md crates/adapters/tardis/README.md
find crates/adapters/databento/test_data -type f | sort
find crates/adapters/tardis/test_data -type f | sort
find crates/adapters/databento/tests crates/adapters/tardis/tests -type f | sort
rg -n '#\\[(rstest|test|tokio::test)' crates/adapters/databento/src crates/adapters/databento/tests | wc -l
rg -n '#\\[(rstest|test|tokio::test)' crates/adapters/tardis/src crates/adapters/tardis/tests | wc -l
```

Targeted source reads:

```bash
sed -n '1,220p' crates/adapters/databento/Cargo.toml
sed -n '1,220p' crates/adapters/databento/src/lib.rs
sed -n '430,930p' crates/adapters/databento/src/data.rs
sed -n '350,690p' crates/adapters/databento/src/historical.rs
sed -n '700,870p' crates/adapters/databento/src/historical.rs
sed -n '880,1030p' crates/adapters/databento/src/live.rs
sed -n '1,220p' crates/adapters/databento/src/enums.rs
sed -n '1,240p' crates/adapters/tardis/Cargo.toml
sed -n '1,220p' crates/adapters/tardis/src/lib.rs
sed -n '420,570p' crates/adapters/tardis/src/data.rs
sed -n '300,410p' crates/adapters/tardis/src/common/parse.rs
sed -n '1,180p' crates/adapters/tardis/src/config.rs
sed -n '1,180p' crates/adapters/tardis/src/machine/mod.rs
sed -n '1,260p' crates/adapters/tardis/src/http/instruments.rs
sed -n '40,260p' crates/adapters/tardis/src/replay.rs
```

Required and final validation:

```bash
scripts/ai/verify_fast.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-013.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- `scripts/ai/verify_fast.sh`: passed; output ended with `== verify_fast complete ==`.
- Final JSON validation for task state and lease: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed with `agentflow role protocol validation passed`.
- `git diff --check`: passed.

## Tests Added Or Updated

No tests were added in RADP-013. This task is an inventory-only adapter parity
task. RADP-014 is the follow-up fixture task for the gaps recorded here.

## Behavior Impact

No runtime behavior changed. No Databento DBN decoder, live feed handler,
historical loader, Tardis Machine replay/stream parser, HTTP bootstrap, CSV
stream, Parquet writer, credential, public API, Python API, PyO3 binding,
Cython surface, Cargo feature, or persistence behavior changed.

The practical impact is release-gate clarity: Databento and Tardis adapter
parity gaps are now explicit follow-up inputs for fixture coverage and closure
decisions.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR documents adapter gaps without
changing runtime behavior or public APIs.

## Gate Status

RADP-013 is medium risk. It creates adapter inventory evidence only; it does not
change adapter runtime behavior or trading semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the inventory file, this evidence file, and the RADP-013 task state/lease
updates. No runtime, persisted data, adapter protocol, schema, Python, PyO3,
Cython, Cargo feature, or public API rollback is required.
