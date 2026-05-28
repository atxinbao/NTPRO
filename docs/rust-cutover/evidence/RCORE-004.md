# RCORE-004 Evidence - Inventory Rust Serialization/Data/Persistence Gaps

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-004
Risk: high

## Summary

Created a docs-only inventory for Rust serialization/data/persistence parity
gaps. The inventory records current Arrow, Cap'n Proto, SBE, data engine,
catalog streaming, Parquet, Feather, DataFusion, object-store, custom data,
Python/PyO3, FFI, and Cython-blocking boundaries.

RCORE-004 is high risk by role policy because serialization, data, and
persistence define the stored and replayed trading data path. This PR must stop
at `REVIEW_REQUIRED`, must not enable auto-merge, and requires Verification &
Release Gatekeeper review before merge.

## Files Changed

- `docs/rust-cutover/inventory/serialization_data_persistence.md`
- `docs/rust-cutover/inventory/README.md`
- `docs/rust-cutover/evidence/RCORE-004.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-004.json`

## Commands Run

```bash
python3 scripts/control/dispatch_next.py --max-risk high
mcp__shrimp_task_manager__.query_task(query="RCORE-004")
mcp__shrimp_task_manager__.list_tasks(status="in_progress")
mcp__code_index__.search_code(directory="/Users/mac/Documents/NTPRO", query="pyo3 OR pyclass OR pymethods OR pyo3_stub_gen file:crates/(serialization|data|persistence)/")
mcp__code_index__.search_code(directory="/Users/mac/Documents/NTPRO", query="TODO OR FIXME OR todo OR panic! file:crates/(serialization|data|persistence)/")
find crates/serialization crates/data crates/persistence -maxdepth 3 -type f
find crates/serialization crates/data crates/persistence -type f -path '*/python/*'
find crates/serialization crates/data crates/persistence -type f -path '*/ffi/*'
find crates/serialization crates/data crates/persistence -type f -path '*/tests/*'
rg -n 'TODO|FIXME' crates/serialization crates/data crates/persistence -g '*.rs' -g 'Cargo.toml'
rg -n 'unsafe' crates/serialization crates/data crates/persistence -g '*.rs'
rg -n 'high-precision|PRECISION_BYTES|PrecisionMismatch|fixed|PriceRaw|QuantityRaw|from_raw_checked|correct_' crates/serialization crates/data crates/persistence -g '*.rs' -g 'Cargo.toml'
rg -n 'not yet stable|may break|TODO|C/Cython blocking|Using FFI API wrapper temporarily|Cannot pass mixed data|not yet implemented' crates/serialization crates/data crates/persistence -g '*.rs' -g 'README.md'
```

Required/final validation commands:

```bash
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RCORE-004.json >/dev/null
git diff --check
```

## Command Results

- `dispatch_next.py --max-risk high`: failed during `git fetch --prune origin`
  because the local machine could not connect to `github.com:443`. The branch,
  lease, agentflow state, and Shrimp status were then created with equivalent
  local steps from already-synced `main` at merge commit `6eb6c582c5`.
- Shrimp isolated queue: confirmed RCORE-004 is the single in-progress task.
- Code-Index: found PyO3/stub surfaces in serialization/data/persistence,
  especially persistence Python and custom-data macro paths.
- Inventory scans found:
  - `serialization=67`, `data=25`, `persistence=24` Rust source files under
    `src`.
  - `serialization=88`, `data=30`, `persistence=31` Rust files across crate
    trees, including tests/benches/generated code.
  - 11 Cap'n Proto schema files and 11 generated Cap'n Proto Rust files.
  - 16 Python-facing source files.
  - 22 Rust files mentioning PyO3/stub generation.
  - 11 integration test files under the three crate `tests/` directories.
  - 10 TODO/FIXME matches.
  - 60 `unsafe` matches.
- `scripts/ai/verify_fast.sh`: passed after inventory/evidence and agentflow
  updates.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow` JSON validation: passed.
- `git diff --check`: passed.

## Tests Added or Updated

No Rust tests were added or updated. RCORE-004 is an inventory task. The
inventory routes test closure to RCORE-005 and implementation/scope closure to
RCORE-006 or dedicated removal gates.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No adapter behavior
changed. No data format, schema, persistence, Cargo feature, Python, PyO3,
Cython, or FFI files were deleted or moved.

## Public API Impact

None. This task adds inventory/evidence documentation only.

## Migration Note Status

No migration note is required because no public API changed. The inventory
states that Python/PyO3/Cython/FFI and data-format removal remain blocked until
dedicated follow-up tasks and release gates approve them.

## Rollback Plan

Revert the inventory document, inventory README entry, evidence file, and the
RCORE-004 `.agentflow` metadata updates. No crate code, build configuration,
schema, catalog data, or runtime data needs rollback.
