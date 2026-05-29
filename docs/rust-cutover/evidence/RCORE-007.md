# RCORE-007 Evidence - Inventory Rust Common Cache/Message Bus/Component Lifecycle Gaps

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-007
Risk: high

## Summary

Created the common-runtime gap inventory for Rust cache, message bus, and
component lifecycle cutover work.

The inventory records existing Rust coverage and remaining release blockers for:

- `Cache` in-memory state, database backing, and snapshot boundaries;
- `MessageBus` typed routing, Any routing, thread-local state, switchboard,
  backing-database facade, and bus tap capture;
- `Component` state transitions, global registry borrow tracking, and
  `UnsafeCell` lifecycle access;
- `Trader`, `NautilusKernel`, backtest, live, data, execution, risk, and
  portfolio lifecycle interaction points;
- active Python/PyO3 wrappers that remain out of scope for RCORE removal.

RCORE-007 is high risk because cache/message-bus/lifecycle ordering affects
runtime determinism and later Rust-only release gates. This PR must stop at
`REVIEW_REQUIRED`, must not enable auto-merge, and requires Verification &
Release Gatekeeper review before merge.

## Files Changed

- `docs/rust-cutover/inventory/common_cache_msgbus_lifecycle.md`
- `docs/rust-cutover/inventory/README.md`
- `docs/rust-cutover/evidence/RCORE-007.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-007.json`

## Commands Run

Dispatch and task management:

```bash
python3 scripts/control/dispatch_next.py --max-risk high
mcp__shrimp_task_manager__.get_task_detail(taskId="e363204c-f231-4866-95ab-70f78b444cca")
mcp__shrimp_task_manager__.process_thought(...)
mcp__shrimp_task_manager__.analyze_task(...)
mcp__shrimp_task_manager__.reflect_task(...)
```

Discovery:

```bash
mcp__code_index__.list_indexes()
mcp__code_index__.search_code(directory="/Users/mac/Documents/NTPRO", query="(Cache|MessageBus|msgbus|ComponentState|Component|lifecycle|start stop) file:crates/(common|core|system|data|execution|risk|portfolio|live|backtest)/.*\\.rs")
rg -n "pub fn (publish|send|request|response)|dispatch_tap|fill_matching|take\\(|replace\\(" crates/common/src/msgbus/api.rs crates/common/src/msgbus/mod.rs crates/common/src/msgbus/core.rs
rg -n "fn (load|flush|dispose|reset|check_integrity|database|snapshot|update|add_|delete_|index_)|self\\.database|CacheSnapshotRef|CacheView" crates/common/src/cache/mod.rs crates/common/src/cache/database.rs crates/common/src/cache/tests.rs
rg -n "(register_message_bus|get_message_bus|set_message_bus|set_bus_tap|clear_bus_tap|dispose\\(|ComponentState|start_component|stop_component|register_component|state\\(|transition_state|initialize\\(|start\\(|stop\\(|dispose\\()" crates/system/src crates/live/src crates/backtest/src crates/data/src crates/execution/src crates/risk/src crates/portfolio/src crates/common/src -g '*.rs'
rg -n "test_.*(start|stop|shutdown|lifecycle|component|msgbus|cache|state|run)|run_|start\\(|stop\\(|shutdown" crates/live/tests/node.rs crates/live/tests/stress.rs crates/system/src/trader.rs crates/common/src/component.rs crates/common/src/msgbus/api.rs crates/common/src/cache/tests.rs
find crates/common/src/cache crates/common/src/msgbus -type f -name '*.rs' -maxdepth 2 | sort
find crates/common/src/python crates/live/src/python crates/backtest/src/python -maxdepth 2 -type f | sort | rg '(cache|msgbus|clock|timer|runtime|node|engine|config|enums)'
```

Required validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_fast.sh
```

Metadata validation:

```bash
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RCORE-007.json >/dev/null
git diff --check
```

## Command Results

- `dispatch_next.py --max-risk high`: dispatched RCORE-007 on branch
  `ai/RCORE-007-inventory-rust-common-cache-message-bus-component-lifecycle-gaps`.
- Shrimp isolated queue: confirmed RCORE-007 is the single in-progress task.
- Shrimp planning: `get_task_detail`, `process_thought`, `analyze_task`, and
  `reflect_task` completed. Shrimp requested `split_tasks`, but the exposed
  Shrimp MCP tool list in this Codex session does not include `split_tasks`;
  no task data was deleted or rewritten.
- Code-Index confirmed `/Users/mac/Documents/NTPRO` is indexed and returned the
  common/runtime files used for inventory orientation. One narrow docs query
  returned no matches, so local `rg` was used for the golden-trace and
  inventory document scan.
- `scripts/ai/verify_fast.sh`: passed. It completed toolchain and rustfmt
  checks, skipped legacy mixed-workspace cargo check unless
  `VERIFY_FAST_CARGO_CHECK=1`, skipped clippy unless `VERIFY_FAST_CLIPPY=1`,
  and printed `verify_fast complete`.
- Metadata validation: passed after evidence/status files were updated.

## Tests Added or Updated

No Rust tests were added or updated in RCORE-007.

This task is inventory-only. RCORE-008 owns the targeted Rust test matrix for
the `CML-*` gaps recorded in
`docs/rust-cutover/inventory/common_cache_msgbus_lifecycle.md`.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No adapter behavior
changed. No cache persistence, message-bus routing, component lifecycle,
database, Cargo feature, Python, PyO3, Cython, generated stub, or FFI file was
deleted or moved.

The new inventory records that `removal_allowed = false` for this area.

## Public API Impact

No public function signatures changed.

## Migration Note Status

No migration note is required because no public API signature, routing
contract, lifecycle behavior, or persisted data format changed.

## Rollback Plan

Remove `docs/rust-cutover/inventory/common_cache_msgbus_lifecycle.md`, remove
its entry from `docs/rust-cutover/inventory/README.md`, remove this evidence
file, and restore the RCORE-007 `.agentflow` metadata. No source rollback,
schema migration, or persisted data migration is required.
