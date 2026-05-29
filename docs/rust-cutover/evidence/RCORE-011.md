# RCORE-011 Evidence - Add Rust Tests for Execution/Risk/Order Lifecycle

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-011
Risk: high

## Summary

Added focused Rust test assertions for the RiskEngine accept/deny routing
boundary identified in RCORE-010.

Implemented:

- Strengthened the accepted submit-order test to assert that the command is
  forwarded through `exec_engine_queue_execute` and no denial/rejection event is
  emitted on `ExecEngine.process`.
- Strengthened the halted trading-state submit-order test to assert that
  `OrderDenied` is emitted and the denied command is not forwarded to execution.
- Added the RCORE-011 test matrix to the execution/risk/order lifecycle
  inventory, including explicit blockers that remain for RCORE-012/RTRACE.

Still explicitly deferred:

- full `order_lifecycle` golden replay through execution/risk;
- matching-engine `PriceType::Mark`, fixed-clock, and ignored contingent-order
  paths;
- order emulator/risk integration;
- reconciliation Python-parity limitation decisions;
- executable position and portfolio/PnL golden traces;
- Python/PyO3/Cython removal.

RCORE-011 is high risk because it touches core risk-engine regression tests and
documents runtime gate coverage. This PR must stop at `REVIEW_REQUIRED`, must
not enable auto-merge, and requires Verification & Release Gatekeeper review
before merge.

## Files Changed

- `crates/risk/tests/risk_engine.rs`
- `docs/rust-cutover/inventory/execution_risk_order_lifecycle.md`
- `docs/rust-cutover/evidence/RCORE-011.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-011.json`

## Commands Run

Task and planning:

```bash
python3 scripts/control/dispatch_next.py --workspace /Users/mac/Documents/NTPRO --max-risk high
mcp__shrimp_task_manager__.process_thought(...)
mcp__shrimp_task_manager__.analyze_task(...)
mcp__shrimp_task_manager__.reflect_task(...)
mcp__code_index__.search_code(...)
```

Focused validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-risk --test risk_engine test_submit_order_with_default_settings_then_sends_to_client -- --exact --nocapture
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-risk --test risk_engine test_submit_order_when_trading_halted_then_denies_order -- --exact --nocapture
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-risk --test risk_engine
```

Required final validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RCORE-011.json
git diff --check
```

## Command Results

- `dispatch_next.py --max-risk high`: dispatched RCORE-011 to
  `ai/RCORE-011-add-rust-tests-for-execution-risk-order-lifecycle`.
- `mcp__code_index__.search_code`: unavailable during this run because the
  transport closed; local `rg`/`sed` was used as fallback.
- `cargo test -p nautilus-risk --test risk_engine test_submit_order_with_default_settings_then_sends_to_client -- --exact --nocapture`:
  passed, 1 test.
- `cargo test -p nautilus-risk --test risk_engine test_submit_order_when_trading_halted_then_denies_order -- --exact --nocapture`:
  passed, 1 test.
- `cargo test -p nautilus-risk --test risk_engine`: passed, 78 passed; 0
  failed; 6 ignored.
- `cargo fmt --check`: passed.
- `python3 -m json.tool .agentflow/state/task_status.json` and
  `python3 -m json.tool .agentflow/leases/RCORE-011.json`: passed.
- `git diff --check`: passed.
- `scripts/ai/verify_full.sh`: passed and completed with
  `== verify_full complete ==`.
- `scripts/ai/verify_fast.sh`: passed and completed with
  `== verify_fast complete ==`.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.

## Tests Added or Updated

- Updated `test_submit_order_with_default_settings_then_sends_to_client` to
  assert accepted risk commands do not emit denial/rejection events.
- Updated `test_submit_order_when_trading_halted_then_denies_order` to assert
  halted risk denial does not forward the command to execution.

No new implementation code was added.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No adapter behavior
changed. No public API changed. No Python, PyO3, Cython, generated stub, or FFI
file was deleted or moved.

## Public API Impact

No public function signatures changed.

## Migration Note Status

No migration note is required because this task only updates tests,
inventory/evidence documentation, and task state.

## Rollback Plan

Revert the two added assertions in `crates/risk/tests/risk_engine.rs`, remove the
RCORE-011 test matrix from the inventory, remove this evidence file, and revert
the RCORE-011 task state/lease updates. No code, schema, persisted data, or
runtime rollback is required.
