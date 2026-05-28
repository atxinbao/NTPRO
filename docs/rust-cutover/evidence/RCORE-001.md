# RCORE-001 Evidence - Inventory Rust Core/Model Value Types Gaps

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-001

## Summary

Created a docs-only inventory for Rust core/model value type parity gaps. The
inventory records current core primitives, model numeric value types,
identifier wrappers, Python/PyO3/Cython/FFI surfaces, precision-mode gaps, and
follow-up ownership for RCORE-002/RCORE-003/RREM tasks.

RCORE-001 is high risk by role policy, so this work must stop at
`REVIEW_REQUIRED` before merge and must not be auto-merged.

## Files Changed

- `docs/rust-cutover/inventory/core_model_value_types.md`
- `docs/rust-cutover/inventory/README.md`
- `docs/rust-cutover/evidence/RCORE-001.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-001.json`

## Commands Run

```bash
mcp__shrimp_task_manager__.list_tasks(status="in_progress")
mcp__shrimp_task_manager__.get_task_detail(taskId="d06c4e1b-9b2b-4011-85be-f5ca68d885f7")
mcp__code_index__.list_indexes()
mcp__code_index__.search_code(directory="/Users/mac/Documents/NTPRO", query="pyo3 file:crates/core/")
mcp__code_index__.search_code(directory="/Users/mac/Documents/NTPRO", query="pyclass file:crates/model/")
rg -n 'cfg_attr\(feature = "python"|pyo3::pyclass|pyo3_stub_gen|cython|cbindgen|cython-compat|high-precision|TODO|panic!' crates/core crates/model -g '*.rs' -g 'Cargo.toml' -g 'build.rs'
find crates/core/src crates/model/src -path '*/python/*' -type f
find crates/core/src crates/model/src -path '*/ffi/*' -type f
rg -n '#\[cfg\(test\)|mod tests|rstest|proptest|serde_json|from_str|from_raw|from_mantissa|high_precision|high-precision|FIXED_PRECISION|MAX_FLOAT_PRECISION' crates/model/src/types/price.rs crates/model/src/types/quantity.rs crates/model/src/types/money.rs crates/model/src/types/currency.rs crates/model/src/types/balance.rs crates/model/src/identifiers/*.rs crates/core/src/nanos.rs crates/core/src/uuid.rs crates/core/src/string/stack_str.rs
```

Final validation commands:

```bash
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RCORE-001.json >/dev/null
git diff --check
```

## Command Results

- Shrimp isolated queue: confirmed one in-progress task, RCORE-001.
- Code-Index: confirmed `/Users/mac/Documents/NTPRO` index exists and found
  PyO3/PyO3 stub surfaces in `crates/core` and `crates/model`.
- Inventory scans: observed 124 `src/python/**` files, 44 `src/ffi/**` files,
  229 source files mentioning PyO3/PyO3 stub annotations, and four cbindgen
  config files for C/Cython generation under `crates/core` and `crates/model`.
- `scripts/ai/verify_fast.sh`: passed; fast mode ran toolchain detection and
  `cargo fmt --check`; cargo check and clippy remained skipped by fast-mode
  defaults.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow` JSON validation: passed.
- `git diff --check`: passed.

## Tests Added or Updated

No Rust tests were added or updated. RCORE-001 is an inventory task. The
inventory explicitly routes test closure to RCORE-002 and implementation/scope
closure to RCORE-003.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No adapter behavior
changed. No Cargo features changed. No Python, PyO3, Cython, or FFI files were
deleted or moved.

## Public API Impact

None. This task adds inventory/evidence documentation only.

## Migration Note Status

No migration note is required because no public API changed. The inventory
states that Python/PyO3/Cython removal remains blocked until dedicated RREM
tasks and release gates approve it.

## Rollback Plan

Revert the inventory document, evidence file, inventory README entry, and the
RCORE-001 `.agentflow` metadata updates. No crate code, build configuration, or
runtime data needs rollback.
