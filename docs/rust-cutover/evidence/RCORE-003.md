# RCORE-003 Evidence - Close Rust Core/Model Value Type Gaps

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-003
Risk: high

## Summary

Closed the remaining raw fixed-point scale validation gap for checked Rust
constructors on `Price`, `Quantity`, and `Money`.

`from_raw_checked` now rejects non-zero raw values that are not aligned to the
declared precision or currency precision. The panicking `from_raw` constructors
remain low-level compatibility entry points because existing arithmetic,
property, max/min, and overflow-boundary tests use arbitrary in-range internal
raw units.

RCORE-003 is high risk by role policy because `Price`, `Quantity`, and `Money`
are core trading value types. This PR must stop at `REVIEW_REQUIRED`, must not
enable auto-merge, and requires Verification & Release Gatekeeper review before
merge.

## Files Changed

- `crates/model/src/types/price.rs`
- `crates/model/src/types/quantity.rs`
- `crates/model/src/types/money.rs`
- `crates/model/tests/value_type_gate.rs`
- `docs/rust-cutover/inventory/core_model_value_types.md`
- `docs/rust-cutover/evidence/RCORE-003.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-003.json`

## Commands Run

```bash
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-model --test value_type_gate
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-model --test value_type_gate --features high-precision,defi
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-model
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo clippy -p nautilus-model --tests --features high-precision,defi -- -D warnings
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_full.sh
```

Final validation commands after evidence and agentflow updates:

```bash
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo fmt --check
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RCORE-003.json >/dev/null
git diff --check
```

## Command Results

- Initial strict `from_raw` attempt: failed `cargo test -p nautilus-model` with
  Money/Price/Quantity max/min/property failures. This confirmed `from_raw`
  is still used as a low-level internal raw constructor and should not be made
  strict in this task without a separate public API and migration gate.
- `cargo test -p nautilus-model --test value_type_gate`: passed, 3 tests.
- `cargo test -p nautilus-model --test value_type_gate --features high-precision,defi`:
  passed, 4 tests.
- `cargo test -p nautilus-model`: passed; model library, integration tests, and
  doc-tests completed.
- `cargo clippy -p nautilus-model --tests --features high-precision,defi -- -D warnings`:
  passed.
- `scripts/ai/verify_full.sh`: passed; full clippy, workspace Rust tests,
  golden trace tests, and rustdoc completed.
- `cargo fmt --check`: passed after evidence and agentflow updates.
- `scripts/ai/verify_fast.sh`: passed after evidence and agentflow updates.
- `scripts/ai/validate_agentflow_roles.py`: passed after evidence and
  agentflow updates.
- `.agentflow` JSON validation: passed.
- `git diff --check`: passed.

## Tests Added or Updated

- Updated `crates/model/tests/value_type_gate.rs` so checked raw constructors
  reject scale-mismatched raw values:
  - `Price::from_raw_checked(1, 0)`
  - `Quantity::from_raw_checked(1, 0)`
  - `Money::from_raw_checked(1, Currency::USD())`
- Updated `test_money_from_raw_checked_valid` to derive the expected raw value
  from `Money::new(123.45, Currency::USD())`, keeping the test valid under both
  standard precision and `high-precision`.

## Behavior Impact

`Price::from_raw_checked`, `Quantity::from_raw_checked`, and
`Money::from_raw_checked` now reject non-zero raw values that do not match the
declared fixed-point scale.

`Price::from_raw`, `Quantity::from_raw`, and `Money::from_raw` remain low-level
raw constructors for existing internal arithmetic and boundary-test use cases.
No trading strategy logic, adapter behavior, persistence format, Cargo feature,
Python, PyO3, Cython, or FFI surface was removed or moved.

## Public API Impact

No function signatures changed.

Checked raw constructor behavior is stricter for invalid raw-scale inputs. Code
that needs validated external or product-surface construction should use
`from_raw_checked`. Code that is intentionally operating on internal raw units
can continue to use `from_raw`.

## Migration Note Status

This evidence file and `docs/rust-cutover/inventory/core_model_value_types.md`
record the migration note for this task: checked raw constructors are strict;
panicking `from_raw` remains a low-level compatibility exception. Any future
attempt to make `from_raw` strict requires a separate public API and migration
gate.

## Rollback Plan

Revert the helper functions and `from_raw_checked` validation changes in
`price.rs`, `quantity.rs`, and `money.rs`; revert the value-type gate test
updates; revert the inventory/evidence updates; and restore the RCORE-003
agentflow metadata. No persisted data migration is required.
