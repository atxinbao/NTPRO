# RPROD-007 Evidence

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-007
Branch: `ai/RPROD-007-define-data-catalog-cli-contract`

## Summary

Defined the Rust-first data/catalog CLI contract in
`docs/rust-cutover/product/DATA_CATALOG_CLI_CONTRACT.md` and synchronized the
top-level Rust CLI contract so `nautilus data` includes inspect, validate, and
load workflows.

RPROD-007 is a product contract task. It does not implement the `data` CLI
command. RPROD-008 owns implementation or blocker handoff for the command
surface.

## Files Changed

- Created `docs/rust-cutover/product/DATA_CATALOG_CLI_CONTRACT.md`.
- Updated `docs/rust-cutover/product/RUST_CLI_CONTRACT.md`.
- Created `docs/rust-cutover/evidence/RPROD-007.md`.
- Updated `.agentflow/state/task_status.json`.
- Updated `.agentflow/leases/RPROD-007.json`.

## Commands Run

```bash
git status --short --branch
sed -n '1,220p' docs/rust-cutover/tasks/RPROD-007.md
sed -n '1,260p' docs/rust-cutover/product/RUST_CLI_CONTRACT.md
sed -n '1,260p' docs/rust-cutover/product/BACKTEST_CLI_CONTRACT.md
sed -n '1,280p' docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md
rg -n "catalog|data" crates/cli/src docs/rust-cutover/product docs/rust-cutover/tasks/RPROD-*.md
rg -n "DataCatalog|ParquetDataCatalog|catalog_path|catalog" crates/data crates/persistence crates/model crates/adapters -g '*.rs'
rg -n "BacktestDataConfig|catalog_path|DataConfig" crates/backtest crates/live crates/data -g '*.rs'
python3 scripts/ai/lease.py claim RPROD-007 --force --branch ai/RPROD-007-define-data-catalog-cli-contract --agent-id Codex --path docs/rust-cutover/tasks/RPROD-007.md --path docs/rust-cutover/product/RUST_CLI_CONTRACT.md --path docs/rust-cutover/product/DATA_CATALOG_CLI_CONTRACT.md --path docs/rust-cutover/evidence/RPROD-007.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-007.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- data --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
```

Code-Index MCP was also used to index `/Users/mac/Documents/NTPRO` and search
for `ParquetDataCatalog`, `BacktestDataConfig`, and `nautilus data`.

## Command Results

- `git status --short --branch`: confirmed the active branch is
  `ai/RPROD-007-define-data-catalog-cli-contract`.
- Task and existing product contract docs were read before editing.
- Local and Code-Index searches confirmed:
  - `crates/cli/src/opt.rs` currently has no `data` command;
  - `ParquetDataCatalog` is the Rust catalog boundary;
  - `BacktestDataConfig` is the Rust data query boundary already used by
    backtest config and `BacktestNode`.
- `lease.py claim`: passed and expanded the RPROD-007 lease to include the new
  product contract and updated top-level CLI contract.
- `cargo run -q -p nautilus-cli -- data --help`: expected blocker; exited with
  code 2 because the current CLI does not expose `data`.
- `scripts/ai/verify_fast.sh`: passed. Fast mode ran toolchain and rustfmt
  checks, then skipped optional workspace cargo check and clippy because their
  opt-in environment variables were not set.

## Tests Added or Updated

No runtime tests were added. RPROD-007 is a product contract task, not a CLI
implementation task.

## Behavior Impact

No runtime behavior changed. The new contract defines expected Rust-first
data/catalog behavior and explicitly records that `nautilus data` is currently
absent from the CLI.

## Public API Impact

No current public API changed. The document defines a future CLI contract:

```text
nautilus data inspect --config <path> [--output <dir>]
nautilus data validate --config <path>
nautilus data load --config <path> [--run-id <id>] [--output <dir>]
```

## Migration Note Status

No migration note is required for this task because no existing CLI behavior was
changed.

## Rollback Plan

- Remove `docs/rust-cutover/product/DATA_CATALOG_CLI_CONTRACT.md`.
- Restore the `nautilus data` section of
  `docs/rust-cutover/product/RUST_CLI_CONTRACT.md`.
- Remove `docs/rust-cutover/evidence/RPROD-007.md`.
- Restore `.agentflow/state/task_status.json` to the previous task state if the
  task is abandoned.
- Release or remove `.agentflow/leases/RPROD-007.json` if abandoning the branch.
