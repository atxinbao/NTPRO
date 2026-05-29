# RCORE-010 Evidence - Inventory Rust Execution/Risk/Order Lifecycle Gaps

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-010
Risk: high

## Summary

Created the execution/risk/order lifecycle inventory for the Rust cutover. The
inventory records current Rust execution and risk surfaces, identifies release
gate blockers, and routes follow-up work to RCORE-011, RCORE-012, RTRACE, RADP,
RREM, and Verification & Release Gatekeeper tasks.

Key findings:

- Rust execution and risk crates have broad local test coverage, but final
  Rust-only release evidence is not complete.
- `order_lifecycle` golden rows exist as schema seed evidence, not full
  execution/risk replay.
- `risk`, `execution`, `position`, and `portfolio_pnl` remain release blockers
  in current golden-trace gate evidence.
- Python/PyO3/Cython surfaces remain active and are not authorized for removal.

RCORE-010 is high risk because this inventory controls later runtime closure and
removal-gate decisions. This PR must stop at `REVIEW_REQUIRED`, must not enable
auto-merge, and requires Verification & Release Gatekeeper review before merge.

## Files Changed

- `docs/rust-cutover/inventory/execution_risk_order_lifecycle.md`
- `docs/rust-cutover/inventory/README.md`
- `docs/rust-cutover/evidence/RCORE-010.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-010.json`

## Commands Run

Task and planning:

```bash
mcp__shrimp_task_manager__.query_task(query="RCORE-010")
mcp__shrimp_task_manager__.process_thought(...)
mcp__shrimp_task_manager__.analyze_task(...)
mcp__shrimp_task_manager__.reflect_task(...)
mcp__code_index__.list_indexes()
mcp__code_index__.search_code(...)
python3 scripts/ai/lease.py claim RCORE-010 --branch ai/RCORE-010-inventory-rust-execution-risk-order-lifecycle-gaps ...
```

Context and evidence collection:

```bash
rg -n "TODO|FIXME|todo!|unimplemented!|panic!|Python|PyO3|Cython|unsafe|stub|temporar|not yet|TBD" crates/execution crates/risk
rg -n "pub fn register_msgbus_handlers|exec_engine_queue_execute|fn determine_position_id|fn flip_position|Python behavior|Equivalent to Python|pub fn process|pub fn execute" crates/execution/src/engine/mod.rs
rg -n "pub fn register_msgbus_handlers|risk_engine_queue_execute|exec_engine_queue_execute|fn create_submit_throttler|fn create_modify_order_throttler|pub fn execute|fn deny_command|fn deny_order|Invalid order side|real-time account balance|TODO" crates/risk/src/engine/mod.rs crates/risk/tests/risk_engine.rs
rg -n "PriceType::Mark|correct clock fixed time|TODO: fix|not yet available|matching Python behavior|Documented limitation shared with Python reference|Cython|Python|TODO|panic!" crates/execution/src/matching_engine/engine.rs crates/execution/src/order_emulator/emulator.rs crates/execution/src/reconciliation/orders.rs crates/execution/src/reconciliation/tests.rs crates/execution/src/matching_engine/config.rs
find tests/golden crates -path '*golden*' -type f | sort
```

Required final validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RCORE-010.json
git diff --check
```

## Command Results

- Shrimp isolated queue marked RCORE-010 as `in_progress`.
- Code-Index was used to locate execution and risk engine surfaces.
- Local scans confirmed:
  - `crates/execution/src` contains 39 Rust source files.
  - `crates/risk/src` contains 6 Rust source files.
  - main execution/risk integration test files contain 779 observed test
    markers.
  - `tests/golden/order_lifecycle_schema.jsonl` contains six schema-level
    order lifecycle cases.
- `scripts/ai/verify_fast.sh`: passed with Rust fmt; cargo check and clippy
  stayed skipped by the script's fast-mode defaults.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- `python3 -m json.tool .agentflow/state/task_status.json`: passed.
- `python3 -m json.tool .agentflow/leases/RCORE-010.json`: passed.
- `git diff --check`: passed.

## Tests Added or Updated

None. RCORE-010 is an inventory task only. RCORE-011 owns adding or identifying
Rust tests for execution/risk/order lifecycle coverage.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No adapter behavior
changed. No public API changed. No Python, PyO3, Cython, generated stub, or FFI
file was deleted or moved.

## Public API Impact

No public function signatures changed.

## Migration Note Status

No migration note is required because this task only adds inventory/evidence
documentation and task state.

## Rollback Plan

Remove `docs/rust-cutover/inventory/execution_risk_order_lifecycle.md`, remove
the README inventory entry, remove this evidence file, and revert the
RCORE-010 task status and lease updates. No code, schema, persisted data, or
runtime rollback is required.
