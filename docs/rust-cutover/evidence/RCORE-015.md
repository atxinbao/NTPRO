# RCORE-015 Evidence - Close Rust Portfolio / Accounting / PnL Gaps

Date: 2026-05-30
Executor: Codex
Task ID: RCORE-015
Risk: high

## Summary

Closed the remaining Rust account calculated-state reporting gap from the
portfolio/accounting/PnL inventory:

- `CashAccount::calculated_account_state` now returns the stored
  `base.calculate_account_state` flag instead of a hard-coded `false`.
- `MarginAccount::calculated_account_state` now returns the stored
  `base.calculate_account_state` flag instead of a hard-coded `false`.
- `AccountAny::set_calculate_account_state` has regression coverage showing the
  concrete account trait result updates for cash, margin, and betting accounts.

The RCORE-015 scoping decision keeps current missing exchange-rate and
missing-balance fallback semantics unchanged until a dedicated typed
accounting-result contract is approved. Golden trace replay remains deferred to
RTRACE/release-gate work, and Python/PyO3/Cython removal remains deferred to the
RREM/removal gate.

## Files Changed

- `crates/model/src/accounts/cash.rs`
- `crates/model/src/accounts/margin.rs`
- `crates/model/src/accounts/any.rs`
- `docs/rust-cutover/inventory/portfolio_accounting_pnl.md`
- `docs/rust-cutover/evidence/RCORE-015.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-015.json`

## Commands Run

Task setup and scope:

```bash
python3 scripts/control/close_merged_pr.py --pr 49 --task-id RCORE-014 --workspace /Users/mac/Documents/NTPRO
python3 scripts/control/dispatch_next.py --max-risk high
rg ...
sed ...
```

Targeted validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo fmt --all --check
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo fmt --all
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-model calculated_account_state
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-model set_calculate_account_state
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-model
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-portfolio
```

Required validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-core --lib --features ffi,python ffi::string::tests::test_pystr_to_string -- --exact --nocapture
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
```

Final local checks:

```bash
python3 scripts/ai/lease.py release RCORE-015 --status PR_READY
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RCORE-015.json
python3 -m json.tool /Users/mac/.codex/shrimp-data/NTPRO/tasks.json
git diff --check
```

## Command Results

- `close_merged_pr.py --pr 49 --task-id RCORE-014`: closed the previously
  merged RCORE-014 Shrimp task and backed up the isolated NTPRO task queue.
- `dispatch_next.py --max-risk high`: dispatched RCORE-015 to
  `ai/RCORE-015-close-rust-portfolio-accounting-pnl-gaps`, created
  `.agentflow/leases/RCORE-015.json`, and marked the isolated NTPRO Shrimp task
  in progress.
- First `cargo fmt --all --check`: failed because the new test signatures needed
  formatting.
- `cargo fmt --all`: passed and formatted the account tests.
- `cargo test -p nautilus-model calculated_account_state`: passed; 2 tests
  passed, 2557 filtered.
- `cargo test -p nautilus-model set_calculate_account_state`: passed; 1 test
  passed, 2558 filtered.
- `cargo test -p nautilus-model`: passed; 2559 library tests, 4
  `tests/value_type_gate.rs` tests, and doc tests completed successfully.
- `cargo test -p nautilus-portfolio`: passed; 22 library tests, 66
  `tests/portfolio.rs` tests, and doc tests completed successfully.
- First `scripts/ai/verify_full.sh`: failed once in unrelated
  `ffi::string::tests::test_pystr_to_string` with `left: "clé"` and
  `right: "test string1"`. No RCORE-015 files touch that FFI string path.
- Targeted rerun of `ffi::string::tests::test_pystr_to_string`: passed; 1 test
  passed.
- Second `scripts/ai/verify_full.sh`: passed with `== verify_full complete ==`;
  workspace formatting, clippy, workspace tests, log-global tests, golden trace
  validation, and Rust docs completed successfully.
- `lease.py release RCORE-015 --status PR_READY`: released the RCORE-015 lease
  as `PR_READY`.
- `validate_agentflow_roles.py`: passed.
- JSON validation for `.agentflow/state/task_status.json`,
  `.agentflow/leases/RCORE-015.json`, and the isolated NTPRO Shrimp queue:
  passed.
- `git diff --check`: passed.

## Tests Added or Updated

Added account calculated-state regression coverage:

- `cash::tests::test_calculated_account_state_reflects_constructor_flag`
- `margin::tests::test_calculated_account_state_reflects_constructor_flag`
- `any::tests::test_set_calculate_account_state_updates_concrete_trait_result`

These tests verify that constructor flags and `AccountAny` mutation are visible
through the concrete `Account::calculated_account_state` trait method.

## Behavior Impact

Runtime PnL, margin, balance, equity, snapshot, adapter, and persistence behavior
is unchanged.

The only behavior change is the Rust account trait reporting surface: cash and
margin accounts now report the existing calculated-state flag correctly instead
of always reporting `false`.

## Public API Impact

No public API shape changed. No method signature, crate feature, Python module,
PyO3 binding, Cython module, packaging path, adapter contract, or persistence
schema changed.

## Migration Note Status

No migration note is required. The PR fixes an internal Rust trait result and
does not change public API shape.

## Gate Status

RCORE-015 is ready for Verification & Release Gatekeeper review. The task state
is `REVIEW_REQUIRED`; the lease is `PR_READY`; auto-merge must not be enabled.

This is high risk because portfolio/accounting/PnL evidence gates release
confidence for balances, margins, realized PnL, unrealized PnL, equity, and
snapshot outputs, even though the code change itself is narrowly scoped.

## Rollback Plan

Revert the cash and margin trait method changes, remove the three regression
tests, revert the inventory update, remove this evidence file, and revert the
RCORE-015 task state/lease updates. No persisted data, adapter, schema, Python,
PyO3, Cython, or public API rollback is required.
