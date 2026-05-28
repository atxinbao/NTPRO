# RPROD-009 Evidence

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-009
Branch: `ai/RPROD-009-add-rust-config-validation-command`

## Summary

Added the shared Rust-first config validation CLI contract and command surface:

```text
nautilus config validate --kind <backtest|sandbox|live|data> --config <path> [--output <dir>]
```

The command parses and exposes help. Its execution path intentionally returns
an owner-visible blocker because a shared Rust TOML config parser and
workflow-specific validation implementation have not been added yet. The
blocker points to `docs/rust-cutover/product/CONFIG_VALIDATION_CLI_CONTRACT.md`.

## Files Changed

- Created `docs/rust-cutover/product/CONFIG_VALIDATION_CLI_CONTRACT.md`.
- Updated `docs/rust-cutover/product/RUST_CLI_CONTRACT.md`.
- Updated `crates/cli/src/opt.rs` to add `config validate` options and parser
  tests.
- Updated `crates/cli/src/lib.rs` to route `config validate` to an explicit
  blocker handler.
- Created `docs/rust-cutover/evidence/RPROD-009.md`.
- Updated `.agentflow/state/task_status.json`.
- Updated `.agentflow/leases/RPROD-009.json`.

## Commands Run

```bash
python3 scripts/ai/lease.py claim RPROD-009 --force --branch ai/RPROD-009-add-rust-config-validation-command --agent-id Codex --path docs/rust-cutover/tasks/RPROD-009.md --path crates/cli/src/opt.rs --path crates/cli/src/lib.rs --path crates/cli/src/bin/cli.rs --path docs/rust-cutover/product/RUST_CLI_CONTRACT.md --path docs/rust-cutover/product/CONFIG_VALIDATION_CLI_CONTRACT.md --path docs/rust-cutover/evidence/RPROD-009.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-009.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo fmt --check -p nautilus-cli
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-cli --lib
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- config --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- config validate --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- config validate --kind backtest --config config/backtest.toml
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- config validate --kind live --config config/live.toml
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
```

Code-Index MCP was used to search for existing `ValueEnum` usage before adding
the CLI enum.

## Command Results

- `lease.py claim`: passed and expanded the RPROD-009 lease to the CLI option,
  CLI routing, product contract, evidence, and agentflow state files.
- `cargo fmt --check -p nautilus-cli`: passed.
- `cargo test -p nautilus-cli --lib`: passed with 17 tests.
- `cargo run -q -p nautilus-cli -- config --help`: passed.
- `cargo run -q -p nautilus-cli -- config validate --help`: passed and listed
  `--kind`, `--config`, and `--output`.
- `cargo run -q -p nautilus-cli -- config validate --kind backtest --config config/backtest.toml`:
  expected blocker; exited with code 1 and reported that `config validate` is
  defined but not implemented yet for kind `Backtest`.
- `cargo run -q -p nautilus-cli -- config validate --kind live --config config/live.toml`:
  expected blocker; exited with code 1 and reported that `config validate` is
  defined but not implemented yet for kind `Live`.
- `scripts/ai/verify_fast.sh`: passed. Fast mode ran toolchain and rustfmt
  checks, then skipped optional workspace cargo check and clippy because their
  opt-in environment variables were not set.

## Tests Added or Updated

Updated `crates/cli/src/opt.rs` parser tests to cover:

- top-level help listing `config`;
- `config` help listing `validate`;
- `config validate` help listing `--kind`, `--config`, and `--output`;
- `config validate --kind backtest --config ... --output ...` parsing.

## Behavior Impact

The CLI now exposes a stable shared Rust-first config validation surface. Help
and argument parsing work locally. Runtime execution still stops before real
config validation and returns an explicit not-implemented blocker, which
prevents accidental acceptance of an unvalidated config path.

## Public API Impact

This PR adds public CLI subcommands:

```text
config validate
```

The new command is intentionally non-executing until a later config parser and
workflow validation task connects it to Rust config models.

## Migration Note Status

No migration note is required for existing users because no existing CLI command
changed behavior. The new command is additive and currently blocker-only.

## Rollback Plan

- Remove the `config` command definitions from `crates/cli/src/opt.rs`.
- Remove `run_config_command` routing from `crates/cli/src/lib.rs`.
- Remove `docs/rust-cutover/product/CONFIG_VALIDATION_CLI_CONTRACT.md`.
- Restore the `nautilus config` section of
  `docs/rust-cutover/product/RUST_CLI_CONTRACT.md`.
- Remove this evidence file.
- Restore `.agentflow/state/task_status.json` and `.agentflow/leases/RPROD-009.json`
  to the previous task state if the task is abandoned.
