# RPROD-003 Evidence

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-003
Branch: `ai/RPROD-003-define-backtest-run-cli-contract`

## Summary

Defined the Rust-first backtest run CLI contract for `nautilus backtest
validate` and `nautilus backtest run`. The task records the current blocker that
the backtest CLI command is not implemented yet.

## Files Changed

- Created `docs/rust-cutover/product/BACKTEST_CLI_CONTRACT.md`.
- Created `docs/rust-cutover/evidence/RPROD-003.md`.
- Updated `.agentflow/state/task_status.json`.
- Updated `.agentflow/leases/RPROD-003.json`.

## Commands Run

```bash
git status --short --branch
python3 scripts/ai/lease.py claim RPROD-003 --force --branch ai/RPROD-003-define-backtest-run-cli-contract --agent-id Codex --path docs/rust-cutover/product/BACKTEST_CLI_CONTRACT.md --path docs/rust-cutover/evidence/RPROD-003.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-003.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- backtest --help
ruby -rjson -e 'JSON.parse(File.read(".agentflow/state/task_status.json")); JSON.parse(File.read(".agentflow/leases/RPROD-003.json")); puts "ok json"'
scripts/ai/validate_agentflow_roles.py
git diff --check
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/lease.py release RPROD-003 --status PR_READY
```

## Command Results

- `git status --short --branch`: confirmed the active branch is
  `ai/RPROD-003-define-backtest-run-cli-contract`.
- `lease.py claim`: expanded the RPROD-003 lease to include the backtest CLI
  contract and evidence files.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- backtest --help`:
  expected blocker, failed because the current CLI does not expose `backtest`.
- JSON parse: passed with `ok json`.
- `scripts/ai/validate_agentflow_roles.py`: passed:
  `agentflow role protocol validation passed`.
- `git diff --check`: passed with no output.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh`:
  passed. Fast mode ran toolchain and rustfmt checks, then skipped optional
  workspace cargo check and clippy because their opt-in environment variables
  were not set.
- `lease.py release`: passed and marked `.agentflow/leases/RPROD-003.json` as
  `PR_READY`.

## Tests Added or Updated

No runtime tests were added. RPROD-003 is a product contract and evidence task,
not a CLI implementation task.

## Behavior Impact

No runtime behavior changed. The new contract defines the expected Rust-first
backtest command behavior and explicitly prohibits Python fallback for the
Rust-first CLI path.

## Public API Impact

No current public API changed. The document defines a future CLI contract for
later implementation.

## Migration Note Status

No migration note is required for this task because no existing CLI behavior was
changed.

## Rollback Plan

- Remove `docs/rust-cutover/product/BACKTEST_CLI_CONTRACT.md`.
- Remove `docs/rust-cutover/evidence/RPROD-003.md`.
- Restore `.agentflow/state/task_status.json` to the previous task state if the
  task is abandoned.
- Release or remove `.agentflow/leases/RPROD-003.json` if abandoning the branch.
