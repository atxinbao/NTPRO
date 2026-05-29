# RCORE-014 Evidence - Rust Portfolio / Accounting / PnL Tests

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-014
Risk: high

## Summary

Added a Rust integration test for the portfolio/accounting/PnL path:

- opens a margin account with USD balance;
- replays a buy fill and verifies commission-adjusted account balance;
- replays a sell fill and closes the position;
- verifies realized PnL, flat unrealized PnL, account equity, and
  `PortfolioSnapshot` totals.

This closes the RCORE-014 requirement for Rust tests covering the identified
portfolio/accounting/PnL gap. It does not create the release-level
`portfolio_pnl` golden trace; that remains RTRACE/release-gate work.

No runtime trading logic was changed. No Python, PyO3, Cython, public API,
adapter, persistence, or schema files were changed.

## Files Changed

- `crates/portfolio/tests/portfolio.rs`
- `docs/rust-cutover/inventory/portfolio_accounting_pnl.md`
- `docs/rust-cutover/evidence/RCORE-014.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-014.json`

## Commands Run

Task setup and scope:

```bash
mcp__shrimp_task_manager__.query_task(...)
mcp__shrimp_task_manager__.get_task_detail(...)
mcp__shrimp_task_manager__.analyze_task(...)
mcp__shrimp_task_manager__.process_thought(...)
python3 scripts/control/dispatch_next.py --max-risk high
python3 scripts/ai/lease.py claim RCORE-014 ...
rg ...
sed ...
```

Targeted validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-portfolio test_order_fill_replay_updates_balance_pnl_equity_and_snapshot
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-portfolio
```

Required validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo fmt
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
```

Final local checks:

```bash
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RCORE-014.json
python3 -m json.tool /Users/mac/.codex/shrimp-data/NTPRO/tasks.json
git diff --check
```

## Command Results

- `dispatch_next.py --max-risk high`: dispatched RCORE-014 to
  `ai/RCORE-014-add-rust-tests-for-portfolio-accounting-pnl`, created
  `.agentflow/leases/RCORE-014.json`, and marked the isolated NTPRO Shrimp task
  in progress.
- `cargo test -p nautilus-portfolio test_order_fill_replay_updates_balance_pnl_equity_and_snapshot`:
  passed; 1 test passed, 65 filtered.
- `cargo test -p nautilus-portfolio`: passed; 22 library tests, 66
  `tests/portfolio.rs` tests, and doc tests completed successfully.
- First `scripts/ai/verify_full.sh`: stopped at `cargo fmt --check` because
  the new test needed formatting.
- `cargo fmt`: passed and formatted the new test.
- Second `scripts/ai/verify_full.sh`: passed with `== verify_full complete ==`;
  workspace formatting, clippy, workspace tests, golden trace tests, and Rust
  docs completed successfully.

## Tests Added or Updated

Added `test_order_fill_replay_updates_balance_pnl_equity_and_snapshot` in
`crates/portfolio/tests/portfolio.rs`.

The test verifies:

- opening fill reduces account balance by the opening commission;
- closing fill applies realized PnL and closing commission;
- realized PnL is `6.0 USD`;
- unrealized PnL is empty or zero after close;
- account equity is `1_000_006.0 USD`;
- `PortfolioSnapshot` carries matching balance, realized PnL, unrealized PnL,
  and total equity.

## Behavior Impact

No runtime behavior changed. This PR only adds test coverage and updates
Rust-cutover documentation/evidence.

## Public API Impact

No public API changed.

## Migration Note Status

No migration note is required. No public API, Python, PyO3, Cython, or package
surface changed.

## Gate Status

RCORE-014 is ready for Verification & Release Gatekeeper review. The task state
is `REVIEW_REQUIRED`; the lease is `PR_READY`; auto-merge must not be enabled.

This is high risk because portfolio/accounting/PnL evidence gates release
confidence for balances, realized PnL, unrealized PnL, equity, and snapshot
outputs.

## Rollback Plan

Revert the RCORE-014 test, revert the inventory note, remove this evidence file,
and revert the RCORE-014 task state/lease updates. No runtime, persisted data,
adapter, schema, Python, PyO3, Cython, or public API rollback is required.
