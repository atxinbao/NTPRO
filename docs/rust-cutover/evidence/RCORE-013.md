# RCORE-013 Evidence - Inventory Rust Portfolio / Accounting / PnL Gaps

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-013
Risk: high

## Summary

Created the Rust portfolio/accounting/PnL gap inventory at
`docs/rust-cutover/inventory/portfolio_accounting_pnl.md`.

The inventory separates current Rust evidence from release blockers:

- Rust already has focused coverage for cash and margin accounts, fill PnL,
  locked balances, margin init/maintenance, realized/unrealized PnL, equity,
  snapshots, missing prices, and account-id filters.
- The final release gate is still blocked because there is no executable
  `portfolio_pnl` golden trace replay covering account balance, margin,
  realized PnL, unrealized PnL, and equity output.
- Python/PyO3/Cython portfolio and accounting surfaces remain present and are
  not authorized for removal by this task.

No runtime code was changed. RCORE-013 is high risk because the inventory
defines what portfolio/accounting/PnL evidence can and cannot be claimed for
the Rust-only release gate. The PR must stop at `REVIEW_REQUIRED`, must not
enable auto-merge, and requires Verification & Release Gatekeeper review.

## Files Changed

- `docs/rust-cutover/inventory/portfolio_accounting_pnl.md`
- `docs/rust-cutover/evidence/RCORE-013.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-013.json`

## Commands Run

Task and inventory:

```bash
mcp__shrimp_task_manager__.list_tasks(...)
mcp__shrimp_task_manager__.update_task(...)
mcp__code_index__.search_code(...)
python3 scripts/control/dispatch_next.py --max-risk high
python3 scripts/ai/lease.py claim RCORE-013 ...
rg ...
find ...
sed ...
```

Required and final validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_fast.sh
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-portfolio
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RCORE-013.json
python3 -m json.tool /Users/mac/.codex/shrimp-data/NTPRO/tasks.json
git diff --check
```

## Command Results

- `mcp__shrimp_task_manager__.list_tasks`: confirmed RCORE-013 was pending and
  RCORE-012 was completed in the isolated NTPRO Shrimp queue.
- `mcp__code_index__.search_code`: failed with `Transport closed`; local
  `rg`/`find`/`sed` was used as the fallback repository search path.
- `python3 scripts/control/dispatch_next.py --max-risk high`: hung in
  `git fetch --prune origin` because GitHub HTTPS fetch did not complete in
  this environment. The hung fetch was stopped and RCORE-013 was manually
  dispatched with the same branch, state, lease, and Shrimp status intent.
- `rg`/`find`/`sed`: confirmed Rust portfolio/accounting/PnL surfaces, test
  coverage, Python/PyO3/Cython residual surfaces, and the missing executable
  `portfolio_pnl` golden trace blocker.
- `scripts/ai/verify_fast.sh`: passed with `== verify_fast complete ==`.
- `cargo test -p nautilus-portfolio`: passed; 22 library tests, 65
  `tests/portfolio.rs` tests, and doc tests completed successfully.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed with
  `agentflow role protocol validation passed`.
- JSON validation: passed for `.agentflow/state/task_status.json`,
  `.agentflow/leases/RCORE-013.json`, and the isolated Shrimp queue
  `/Users/mac/.codex/shrimp-data/NTPRO/tasks.json`.
- `git diff --check`: passed.

## Tests Added or Updated

No tests were added or updated. RCORE-013 is an inventory task. Follow-up
RCORE-014 owns adding or identifying Rust tests for the listed gaps.

## Behavior Impact

No trading behavior changed. No account, margin, PnL, equity, snapshot,
adapter, persistence, public API, Python, PyO3, or Cython behavior changed.

## Public API Impact

No public function signatures changed.

## Migration Note Status

No migration note is required because this task only adds inventory and
evidence. Python/PyO3/Cython removal remains blocked by RREM/release gates.

## Gate Status

RCORE-013 is ready for Verification & Release Gatekeeper review. The task state
is `REVIEW_REQUIRED`; the lease is `PR_READY`; auto-merge must not be enabled.

## Rollback Plan

Revert the RCORE-013 inventory file, remove this evidence file, and revert the
RCORE-013 task state/lease updates. No runtime, schema, persisted data,
adapter, or public API rollback is required.
