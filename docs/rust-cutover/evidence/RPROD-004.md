# RPROD-004 Evidence

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-004
Branch: `ai/RPROD-004-implement-or-close-backtest-cli-entrypoint-gaps`

## Summary

Implemented the Rust CLI backtest entrypoint shape for `nautilus backtest
validate` and `nautilus backtest run`. The command surface and help now exist,
while actual validation and runtime execution return an explicit blocker until a
later scoped task adds the config parser, strategy registry, and runtime smoke.

## Files Changed

- Updated `crates/cli/src/opt.rs`.
- Updated `crates/cli/src/lib.rs`.
- Updated `crates/cli/src/bin/cli.rs`.
- Updated `scripts/ai/verify_full.sh`.
- Updated `scripts/ai/run_golden_traces.sh`.
- Created `docs/rust-cutover/evidence/RPROD-004.md`.
- Updated `.agentflow/state/task_status.json`.
- Updated `.agentflow/leases/RPROD-004.json`.

## Commands Run

```bash
git status --short --branch
sed -n '1,220p' docs/rust-cutover/tasks/RPROD-004.md
python3 scripts/ai/lease.py claim RPROD-004 --force --branch ai/RPROD-004-implement-or-close-backtest-cli-entrypoint-gaps --agent-id Codex --path crates/cli/src/opt.rs --path crates/cli/src/lib.rs --path crates/cli/src/bin/cli.rs --path docs/rust-cutover/evidence/RPROD-004.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-004.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo fmt --check -p nautilus-cli
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-cli --lib
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- backtest --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- backtest validate --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- backtest run --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- backtest validate --config config/backtest.toml
bash -n scripts/ai/run_golden_traces.sh scripts/ai/verify_full.sh
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/run_golden_traces.sh
scripts/ai/validate_agentflow_roles.py
git diff --check
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_full.sh
python3 scripts/ai/lease.py release RPROD-004 --status PR_READY
```

## Command Results

- `lease.py claim`: expanded the RPROD-004 lease to include the CLI files,
  evidence, state, and lease file.
- `cargo fmt --check -p nautilus-cli`: passed.
- `cargo test -p nautilus-cli --lib`: passed; 4 CLI parser tests passed.
- `cargo run -q -p nautilus-cli -- backtest --help`: passed and listed
  `validate` and `run`.
- `cargo run -q -p nautilus-cli -- backtest validate --help`: passed and
  documented `--config <CONFIG>`.
- `cargo run -q -p nautilus-cli -- backtest run --help`: passed and documented
  `--config`, `--run-id`, and `--output`.
- `cargo run -q -p nautilus-cli -- backtest validate --config config/backtest.toml`:
  failed with exit code 1 as expected, with an explicit not-implemented blocker
  pointing to `docs/rust-cutover/product/BACKTEST_CLI_CONTRACT.md`.
- `bash -n scripts/ai/run_golden_traces.sh scripts/ai/verify_full.sh`: passed.
- `scripts/ai/run_golden_traces.sh`: passed; validated
  `tests/golden/schema_smoke.jsonl`.
- `scripts/ai/verify_full.sh`: passed after local fallback compatibility fixes;
  fast checks, clippy, workspace Rust tests, isolated log-global tests, golden
  trace validation, and Rust docs completed successfully.
- `scripts/ai/validate_agentflow_roles.py`: passed:
  `agentflow role protocol validation passed`.
- `git diff --check`: passed with no output.
- `lease.py release`: passed and marked `.agentflow/leases/RPROD-004.json` as
  `PR_READY`.

## Local Verification Compatibility

The initial full verification run exposed two local tooling gaps rather than
RPROD-004 CLI regressions:

- `scripts/ai/verify_full.sh` fell back to `cargo test` when `cargo-nextest`
  was unavailable. The fallback now skips process-global logger tests from the
  bulk run and reruns them individually, preserving the isolation normally
  provided by nextest.
- `scripts/ai/run_golden_traces.sh` assumed a `python` executable. It now uses
  `PYTHON_BIN` when supplied, otherwise prefers `python3` and then `python`.

## Tests Added or Updated

Added `crates/cli/src/opt.rs` unit tests for:

- top-level help listing `backtest`;
- `backtest` help listing `validate` and `run`;
- `backtest validate --config <path>` parsing;
- `backtest run --config <path> --run-id <id> --output <dir>` parsing.

## Behavior Impact

- `nautilus backtest --help` now succeeds instead of failing as an unknown
  subcommand.
- `nautilus backtest validate --help` and `nautilus backtest run --help` now
  expose the Rust-first contract.
- Running `validate` or `run` returns an explicit blocker. No Python fallback,
  PyO3 dependency, Cython build path, or runtime backtest execution was added.
- CLI execution errors now exit non-zero so automation can detect blockers.

## Public API Impact

This adds a new Rust CLI command surface:

```text
nautilus backtest validate --config <path>
nautilus backtest run --config <path> [--run-id <id>] [--output <dir>]
```

The execution path is intentionally blocked until later implementation tasks
provide config parsing and runtime smoke evidence.

## Migration Note Status

No migration note is required. Existing commands remain available. The change
adds a new command surface and fixes CLI error exit behavior.

## Rollback Plan

- Remove the `Backtest` command types from `crates/cli/src/opt.rs`.
- Remove `run_backtest_command` and the backtest match arm from
  `crates/cli/src/lib.rs`.
- Restore `crates/cli/src/bin/cli.rs` if non-zero error exits must be reverted.
- Remove `docs/rust-cutover/evidence/RPROD-004.md`.
- Restore `.agentflow/state/task_status.json` and release or remove
  `.agentflow/leases/RPROD-004.json` if abandoning the branch.
