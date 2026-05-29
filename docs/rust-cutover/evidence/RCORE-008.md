# RCORE-008 Evidence - Add Rust Tests for Common Cache/Message Bus/Component Lifecycle

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-008
Risk: high

## Summary

Added focused Rust unit coverage for the common cache, message bus, and
component lifecycle gaps inventoried by RCORE-007.

This task adds tests for:

- cache position snapshot persistence blocker visibility;
- message-bus typed route and Any route separation;
- thread-local message-bus lifecycle isolation;
- thread-local component registry lifecycle isolation.

RCORE-008 is high risk because common cache, message-bus routing, and component
lifecycle state sit under Rust runtime determinism. This PR must stop at
`REVIEW_REQUIRED`, must not enable auto-merge, and requires Verification &
Release Gatekeeper review before merge.

## Files Changed

- `crates/common/src/cache/tests.rs`
- `crates/common/src/component.rs`
- `crates/common/src/msgbus/api.rs`
- `docs/rust-cutover/inventory/common_cache_msgbus_lifecycle.md`
- `docs/rust-cutover/evidence/RCORE-008.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-008.json`

## Commands Run

Task and metadata checks:

```bash
mcp__shrimp_task_manager__.list_tasks(status="in_progress")
mcp__shrimp_task_manager__.execute_task(taskId="83b01901-d832-4873-9fdf-74da27b77b92")
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

Focused Rust tests:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-common --lib test_route_separation
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-common --lib test_message_bus_thread_local_isolation_for_lifecycle_state
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-common --lib test_component_registry_is_thread_local_for_lifecycle_isolation
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-common --lib test_snapshot_position_state_release_blocker_is_explicit
```

Formatting and full verification:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" rustup run 1.95.0-aarch64-apple-darwin cargo fmt --check
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
```

## Command Results

- Shrimp isolated queue confirmed RCORE-008 was the in-progress NTPRO task.
- `cargo fmt --check`: passed.
- `cargo test -p nautilus-common --lib test_route_separation`: passed, 2 tests.
- `cargo test -p nautilus-common --lib test_message_bus_thread_local_isolation_for_lifecycle_state`: passed, 1 test.
- `cargo test -p nautilus-common --lib test_component_registry_is_thread_local_for_lifecycle_isolation`: passed, 1 test.
- `cargo test -p nautilus-common --lib test_snapshot_position_state_release_blocker_is_explicit`: passed, 1 test.
- `git diff --check`: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- `scripts/ai/verify_full.sh`: passed. It completed fast verification,
  workspace clippy/test coverage, log-global tests, golden trace validation,
  backtest replay, live sandbox lifecycle replay, adapter payload replay, and
  Rust docs generation.

## Tests Added or Updated

- `cache::tests::test_snapshot_position_state_release_blocker_is_explicit`
  keeps the current `snapshot_position_state` `todo!()` blocker explicit.
- `msgbus::api::tests::test_route_separation_any_subscriber_does_not_receive_typed_quote`
  pins that typed quote publishing does not deliver to an Any route subscriber.
- `msgbus::api::tests::test_route_separation_typed_subscriber_does_not_receive_any_quote`
  pins that Any publishing does not deliver to a typed quote subscriber.
- `msgbus::api::tests::test_message_bus_thread_local_isolation_for_lifecycle_state`
  confirms a custom main-thread bus and subscriptions do not leak to a worker
  thread.
- `component::tests::test_component_registry_is_thread_local_for_lifecycle_isolation`
  confirms a registered component remains isolated to the creating thread.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No adapter behavior
changed. No cache persistence, message-bus routing, component lifecycle,
database, Cargo feature, Python, PyO3, Cython, generated stub, or FFI file was
deleted or moved.

The cache snapshot position state test intentionally records the current
release blocker instead of claiming persistence is complete.

## Public API Impact

No public function signatures changed.

## Migration Note Status

No migration note is required because no public API signature, routing
contract, lifecycle behavior, or persisted data format changed.

## Rollback Plan

Remove the RCORE-008 tests from `crates/common/src/cache/tests.rs`,
`crates/common/src/component.rs`, and `crates/common/src/msgbus/api.rs`;
remove the RCORE-008 matrix from
`docs/rust-cutover/inventory/common_cache_msgbus_lifecycle.md`; remove this
evidence file; and restore the RCORE-008 `.agentflow` metadata. No schema
migration or persisted data rollback is required.
