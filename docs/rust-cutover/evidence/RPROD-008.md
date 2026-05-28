# RPROD-008 Evidence

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-008
Branch: `ai/RPROD-008-implement-or-close-data-catalog-cli-gaps`

## Summary

Implemented the Rust-first `data` CLI entrypoint contract from
`docs/rust-cutover/product/DATA_CATALOG_CLI_CONTRACT.md`.

The commands now parse and expose help for:

```text
nautilus data inspect --config <path> [--output <dir>]
nautilus data validate --config <path>
nautilus data load --config <path> [--run-id <id>] [--output <dir>]
```

The execution path intentionally returns owner-visible blockers because the
task scope is product-surface entrypoints, not catalog runtime wiring. Each
blocker points to the data/catalog CLI contract instead of silently accepting
an unimplemented inspect, validate, or load path.

## Files Changed

- Updated `crates/cli/src/opt.rs` to add the `data` command, subcommand
  options, and parser tests.
- Updated `crates/cli/src/lib.rs` to route `data` commands to explicit blocker
  handlers.
- Created `docs/rust-cutover/evidence/RPROD-008.md`.
- Updated `.agentflow/state/task_status.json`.
- Updated `.agentflow/leases/RPROD-008.json`.

## Commands Run

```bash
python3 scripts/ai/lease.py claim RPROD-008 --force --branch ai/RPROD-008-implement-or-close-data-catalog-cli-gaps --agent-id Codex --path docs/rust-cutover/tasks/RPROD-008.md --path crates/cli/src/opt.rs --path crates/cli/src/lib.rs --path crates/cli/src/bin/cli.rs --path docs/rust-cutover/evidence/RPROD-008.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-008.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo fmt --check -p nautilus-cli
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-cli --lib
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- data --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- data inspect --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- data validate --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- data load --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- data inspect --config config/data.toml
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- data validate --config config/data.toml
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- data load --config config/data.toml
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_full.sh
```

## Command Results

- `lease.py claim`: passed and expanded the RPROD-008 lease to the CLI option,
  CLI routing, evidence, and agentflow state files.
- `cargo fmt --check -p nautilus-cli`: passed.
- `cargo test -p nautilus-cli --lib`: passed with 14 tests.
- `cargo run -q -p nautilus-cli -- data --help`: passed.
- `cargo run -q -p nautilus-cli -- data inspect --help`: passed.
- `cargo run -q -p nautilus-cli -- data validate --help`: passed.
- `cargo run -q -p nautilus-cli -- data load --help`: passed.
- `cargo run -q -p nautilus-cli -- data inspect --config config/data.toml`:
  expected blocker; exited with code 1 and reported that `data inspect` is
  defined but not implemented yet.
- `cargo run -q -p nautilus-cli -- data validate --config config/data.toml`:
  expected blocker; exited with code 1 and reported that `data validate` is
  defined but not implemented yet.
- `cargo run -q -p nautilus-cli -- data load --config config/data.toml`:
  expected blocker; exited with code 1 and reported that `data load` is defined
  but not implemented yet.
- `scripts/ai/verify_full.sh`: passed. Full mode ran fast checks, clippy,
  workspace Rust tests, isolated log-global tests, golden trace validation, and
  Rust docs.

## Tests Added or Updated

Updated `crates/cli/src/opt.rs` parser tests to cover:

- top-level help listing `backtest`, `sandbox`, `live`, and `data`;
- `data` help listing `inspect`, `validate`, and `load`;
- `data inspect` config and output parsing;
- `data validate` config parsing;
- `data load` config, run-id, and output parsing.

## Behavior Impact

The CLI now exposes a stable Rust-first `data` command surface. Help and
argument parsing work locally. Runtime execution still stops before catalog
inspection, validation, or loading and returns an explicit not-implemented
blocker, which prevents accidental use of an incomplete product path.

## Public API Impact

This PR adds public CLI subcommands:

```text
data inspect
data validate
data load
```

The new commands are intentionally non-executing until a later runtime/catalog
wiring task connects them to validated Rust catalog flows.

## Migration Note Status

No migration note is required for existing users because no existing CLI command
changed behavior. The new commands are additive and currently blocker-only.

## Rollback Plan

- Remove the `data` command definitions from `crates/cli/src/opt.rs`.
- Remove `run_data_command` routing from `crates/cli/src/lib.rs`.
- Remove this evidence file.
- Restore `.agentflow/state/task_status.json` and `.agentflow/leases/RPROD-008.json`
  to the previous task state if the task is abandoned.
