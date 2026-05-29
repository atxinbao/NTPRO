# RCORE-009 Evidence - Close Rust Common Cache/Message Bus/Component Lifecycle Gaps

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-009
Risk: high

## Summary

Closed the common-runtime gaps that can be resolved inside the Rust core
runtime boundary and recorded explicit deferrals for the gaps that require
database, integration, release, or removal gates.

Implemented:

- `Cache::snapshot_position_state` now returns `Ok(())` after the existing
  optional database snapshot path or no-database warning, instead of ending in
  `todo!()`.
- Added Rust tests for the no-database and database-adapter
  `snapshot_position_state` paths.
- Added an executable `cache_msgbus` golden trace fixture and Rust replay test
  for cache quote storage, typed message-bus publish ordering, BusTap capture
  before subscriber delivery, and common object dispose state.
- Added the `cache_msgbus` replay to `scripts/ai/run_golden_traces.sh`.
- Updated the common lifecycle inventory and golden-trace gate evidence with
  RCORE-009 closure/deferral decisions.

Still explicitly deferred:

- durable cache/message-bus backing store fixtures;
- full `NautilusKernel`/event-store shutdown ordering trace;
- heavy live stress/performance evidence;
- Python/PyO3/Cython removal.

RCORE-009 is high risk because it touches common runtime lifecycle behavior and
the golden trace gate. This PR must stop at `REVIEW_REQUIRED`, must not enable
auto-merge, and requires Verification & Release Gatekeeper review before merge.

## Files Changed

- `crates/common/src/cache/mod.rs`
- `crates/common/src/cache/tests.rs`
- `crates/common/tests/golden_trace_cache_msgbus.rs`
- `tests/golden/cache_msgbus_schema.jsonl`
- `scripts/ai/run_golden_traces.sh`
- `docs/rust-cutover/golden_trace/GATE_EVIDENCE.md`
- `docs/rust-cutover/golden_trace/SCHEMA.md`
- `docs/rust-cutover/inventory/common_cache_msgbus_lifecycle.md`
- `docs/rust-cutover/evidence/RCORE-009.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-009.json`

## Commands Run

Task and planning:

```bash
mcp__shrimp_task_manager__.execute_task(taskId="ccc66be6-0565-47cf-924a-e79918dd8581")
mcp__shrimp_task_manager__.process_thought(...)
mcp__shrimp_task_manager__.plan_task(...)
mcp__shrimp_task_manager__.analyze_task(...)
mcp__shrimp_task_manager__.reflect_task(...)
mcp__code_index__.list_indexes()
mcp__code_index__.search_code(...)
```

Focused validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" rustup run 1.95.0-aarch64-apple-darwin cargo fmt --check
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-common --lib test_snapshot_position_state
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-common --test golden_trace_cache_msgbus
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" rustup run 1.95.0-aarch64-apple-darwin cargo clippy -p nautilus-common --test golden_trace_cache_msgbus -- -D warnings
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/run_golden_traces.sh
```

Required final validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-core --features "ffi python" --lib ffi::string::tests::test_pystr_to_string -- --exact --nocapture
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" RUST_TEST_THREADS=1 cargo test -p nautilus-core --features "ffi python" --lib
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo fmt --check
git diff --check
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RCORE-009.json
```

## Command Results

- Shrimp isolated queue marked RCORE-009 as `in_progress`.
- Code-Index confirmed `/Users/mac/Documents/NTPRO` is indexed and was used to
  locate cache, message-bus, and golden-trace surfaces.
- `cargo fmt --check`: passed after formatting the new Rust test.
- `cargo test -p nautilus-common --lib test_snapshot_position_state`: passed,
  2 tests.
- `cargo test -p nautilus-common --test golden_trace_cache_msgbus`: passed, 1
  test.
- `cargo clippy -p nautilus-common --test golden_trace_cache_msgbus -- -D warnings`:
  passed after replacing an unnecessary `to_string`.
- `scripts/ai/run_golden_traces.sh`: passed with 7 JSONL files and the new
  `nautilus-common` `cache_msgbus` replay.
- `scripts/ai/verify_full.sh`: failed in the unrelated Python/FFI
  `nautilus-core` test `ffi::string::tests::test_pystr_to_string` during
  parallel full-workspace execution after clippy and many workspace tests had
  passed. The failure was `left: "clé"`, `right: "test string1"`.
- `cargo test -p nautilus-core --features "ffi python" --lib ffi::string::tests::test_pystr_to_string -- --exact --nocapture`:
  passed, 1 test.
- `RUST_TEST_THREADS=1 cargo test -p nautilus-core --features "ffi python" --lib`:
  passed, 997 tests. This isolates the full-run failure as a Python/FFI
  parallel-test issue outside the RCORE-009 cache/message-bus path. RCORE-009
  does not modify Python, PyO3, Cython, or FFI code.
- final `cargo fmt --check`: passed.
- final `git diff --check`: passed.
- final `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- final JSON validation for `.agentflow/state/task_status.json` and
  `.agentflow/leases/RCORE-009.json`: passed.

## Tests Added or Updated

- Updated `cache::tests::test_snapshot_position_state_without_database_returns_ok`.
- Added `cache::tests::test_snapshot_position_state_with_database_returns_ok`.
- Added `crates/common/tests/golden_trace_cache_msgbus.rs` with
  `rust_common_cache_msgbus_replays_quote_ordering_golden_trace`.
- Added `tests/golden/cache_msgbus_schema.jsonl`.
- Updated `scripts/ai/run_golden_traces.sh` to run the new common
  `cache_msgbus` replay by default.

## Behavior Impact

`Cache::snapshot_position_state` no longer panics after completing the existing
optional database snapshot path. Without a database it keeps the existing warning
behavior and returns `Ok(())`. With a database it propagates adapter errors and
returns `Ok(())` on adapter success.

No trading semantics changed. No adapter behavior changed. No persisted schema
or durable database format changed. No Python, PyO3, Cython, generated stub, or
FFI file was deleted or moved.

## Public API Impact

No public function signatures changed.

The observable behavior of `Cache::snapshot_position_state` changed from
unconditional `todo!()` panic to the documented `anyhow::Result<()>` contract.

## Migration Note Status

No migration note is required because no public signature, persisted data
format, or user-facing product API changed.

## Rollback Plan

Restore the `snapshot_position_state` trailing `todo!()` behavior, restore the
old blocker test, remove `crates/common/tests/golden_trace_cache_msgbus.rs`,
remove `tests/golden/cache_msgbus_schema.jsonl`, remove the
`run_golden_traces.sh` common replay invocation, revert the RCORE-009 inventory
and golden-trace documentation updates, and remove this evidence file. No
schema migration or persisted data rollback is required.
