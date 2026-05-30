# RBTL-008 Evidence

Date: 2026-05-30
Executor: Codex

## Task ID

RBTL-008

## Summary

Added a Rust-only sandbox execution smoke test for `nautilus-live`. The test builds a
`LiveNode` in `Environment::Sandbox`, registers `SandboxExecutionClientFactory`
through the Rust builder, starts the node, verifies the sandbox execution client
connects and writes the starting account into cache, then stops the node.

## Files changed

- `crates/live/Cargo.toml`
- `Cargo.lock`
- `crates/live/tests/node.rs`
- `.agentflow/leases/RBTL-008.json`
- `.agentflow/state/task_status.json`
- `docs/rust-cutover/evidence/RBTL-008.md`

## Commands run

- `cargo fmt --check`
- `cargo test -p nautilus-live --test node serial_tests::test_rust_sandbox_execution_client_start_stop_smoke_without_python -- --exact --nocapture`
- `env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-live --test node serial_tests::test_rust_sandbox_execution_client_start_stop_smoke_without_python -- --exact --nocapture`
- `env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc cargo test -p nautilus-live --test node -- --test-threads=1`
- `env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc scripts/ai/verify_full.sh`

## Command results

- `cargo fmt --check`: passed.
- Direct targeted test with the default Homebrew Rust toolchain failed before
  building because that toolchain is `rustc 1.87.0`, while the workspace now
  requires `rustc 1.95.0`.
- Targeted sandbox execution smoke with pinned `rustc 1.95.0`: passed, `1
  passed; 0 failed`.
- Full `crates/live/tests/node.rs` integration suite with pinned `rustc
  1.95.0`: passed, `29 passed; 0 failed`.
- `scripts/ai/verify_full.sh` with pinned `rustc 1.95.0`: passed, including
  golden trace validation and Rust docs.

## Tests added/updated

- Added `serial_tests::test_rust_sandbox_execution_client_start_stop_smoke_without_python`.

## Behavior impact

No production behavior change. This only adds test coverage and a dev dependency
from `nautilus-live` tests to the existing Rust sandbox adapter.

The smoke does not submit orders, match orders, call real venues, or change live
trading semantics. It only proves that the Rust live node can register, start,
observe, and stop the existing sandbox execution client without Python.

## Public API impact

None.

## Migration note status

Not required because this is test-only coverage.

## Rollback plan

Revert the `nautilus-live` dev dependency, the `Cargo.lock` update, and the
added smoke test.
