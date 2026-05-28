# RCORE-002 Evidence - Add Rust Tests for Core/Model Value Types

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-002

## Summary

Added Rust integration tests that lock down public core/model value-type
contracts for timestamp parsing, UUID version validation, stack-string storage,
numeric precision, sentinel rejection, identifier round trips, and DeFi wei
quantity boundaries.

RCORE-002 is high risk by role policy because it touches core/model value-type
contracts. This PR must stop at `REVIEW_REQUIRED`, must not be auto-merged, and
requires Verification & Release Gatekeeper review before merge.

## Files Changed

- `crates/core/tests/value_type_gate.rs`
- `crates/model/tests/value_type_gate.rs`
- `crates/testkit/tests/golden_trace_schema.rs`
- `docs/rust-cutover/evidence/RCORE-002.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-002.json`

## Commands Run

```bash
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo fmt --check
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-core --test value_type_gate
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-model --test value_type_gate
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-model --test value_type_gate --features high-precision,defi
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo clippy -p nautilus-core -p nautilus-model --tests --features high-precision,defi -- -D warnings
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-testkit --test golden_trace_schema
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo clippy -p nautilus-testkit --test golden_trace_schema -- -D warnings
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_full.sh
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RCORE-002.json >/dev/null
git diff --check
```

## Command Results

- `cargo fmt --check`: passed.
- `cargo test -p nautilus-core --test value_type_gate`: passed, 3 tests.
- `cargo test -p nautilus-model --test value_type_gate`: passed, 3 tests.
- `cargo test -p nautilus-model --test value_type_gate --features high-precision,defi`: passed, 4 tests.
- `cargo clippy -p nautilus-core -p nautilus-model --tests --features high-precision,defi -- -D warnings`: passed.
- `cargo test -p nautilus-testkit --test golden_trace_schema`: passed, 1 test.
- `cargo clippy -p nautilus-testkit --test golden_trace_schema -- -D warnings`: passed.
- `scripts/ai/verify_full.sh`: passed; full clippy, workspace Rust tests, golden trace tests, and rustdoc completed.
- `scripts/ai/verify_fast.sh`: passed after evidence/state updates.
- `scripts/ai/validate_agentflow_roles.py`: passed after evidence/state updates.
- `.agentflow` JSON validation: passed.
- `git diff --check`: passed.

## Tests Added or Updated

- Added `crates/core/tests/value_type_gate.rs`:
  - `UnixNanos` numeric/RFC3339/date parsing, negative rejection, and serde round trip.
  - `UUID4` v4 parsing, non-v4 rejection, display, and serde round trip.
  - `StackStr` checked constructor, ASCII/empty rejection, and serde round trip.
- Added `crates/model/tests/value_type_gate.rs`:
  - `Price`, `Quantity`, and `Money` precision/display/serde contracts.
  - `PRICE_UNDEF` and `QUANTITY_UNDEF` checked-constructor sentinel rejection.
  - `InstrumentId`, `ClientOrderId`, and `StrategyId` parse/display/serde contracts.
  - `Quantity::from_wei`/`as_wei` DeFi 18-decimal boundary under the `defi` feature.
- Updated `crates/testkit/tests/golden_trace_schema.rs` with a mechanical clippy
  pattern fix from `Some(case_id) if case_id.is_empty()` to `Some("")`; this
  does not change test semantics and unblocks the required full validation.

## Behavior Impact

No production runtime behavior changed. No trading semantics changed. No adapter
behavior changed. No Cargo features changed. No Python, PyO3, Cython, or FFI
files were deleted or moved.

## Public API Impact

None. The task adds tests for existing public value-type behavior but does not
change public APIs.

## Migration Note Status

No migration note is required because no public API changed.

## Rollback Plan

Revert the two new value-type test files, the narrow `golden_trace_schema`
clippy pattern fix, this evidence file, and the RCORE-002 `.agentflow` metadata
updates. No production runtime code or persisted data needs rollback.
