# RPROD-006 Evidence

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-006
Branch: `ai/RPROD-006-implement-or-close-live-sandbox-cli-entrypoint-gaps`

## Summary

Implemented the Rust-first `sandbox` and `live` CLI entrypoint contracts from
`docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md`.

The commands now parse and expose help for:

```text
nautilus sandbox validate --config <path>
nautilus sandbox run --config <path> [--run-id <id>] [--output <dir>]
nautilus live validate --config <path>
nautilus live run --config <path> [--run-id <id>] [--output <dir>]
```

The execution path intentionally returns owner-visible blockers because the
task scope is product-surface entrypoints, not runtime wiring. Each blocker
points to the live/sandbox CLI contract instead of silently accepting an
unimplemented run path.

## Files Changed

- Updated `crates/cli/src/opt.rs` to add `sandbox` and `live` subcommands,
  command options, and parser tests.
- Updated `crates/cli/src/lib.rs` to route `sandbox` and `live` commands to
  explicit blocker handlers.
- Created `docs/rust-cutover/evidence/RPROD-006.md`.
- Updated `.agentflow/state/task_status.json`.
- Updated `.agentflow/leases/RPROD-006.json`.

## Commands Run

```bash
python3 scripts/ai/lease.py claim RPROD-006 --force --branch ai/RPROD-006-implement-or-close-live-sandbox-cli-entrypoint-gaps --agent-id Codex --path docs/rust-cutover/tasks/RPROD-006.md --path crates/cli/src/opt.rs --path crates/cli/src/lib.rs --path crates/cli/src/bin/cli.rs --path docs/rust-cutover/evidence/RPROD-006.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-006.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo fmt --check -p nautilus-cli
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-cli --lib
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- sandbox --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- sandbox validate --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- sandbox run --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- live --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- live validate --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- live run --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- sandbox validate --config config/sandbox.toml
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- live validate --config config/live.toml
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_full.sh
```

## Command Results

- `lease.py claim`: passed and expanded the RPROD-006 lease to the CLI option,
  CLI routing, evidence, and agentflow state files.
- `cargo fmt --check -p nautilus-cli`: passed.
- `cargo test -p nautilus-cli --lib`: passed with 10 tests.
- `cargo run -q -p nautilus-cli -- sandbox --help`: passed.
- `cargo run -q -p nautilus-cli -- sandbox validate --help`: passed.
- `cargo run -q -p nautilus-cli -- sandbox run --help`: passed.
- `cargo run -q -p nautilus-cli -- live --help`: passed.
- `cargo run -q -p nautilus-cli -- live validate --help`: passed.
- `cargo run -q -p nautilus-cli -- live run --help`: passed.
- `cargo run -q -p nautilus-cli -- sandbox validate --config config/sandbox.toml`:
  expected blocker; exited with code 1 and reported that `sandbox validate` is
  defined but not implemented yet.
- `cargo run -q -p nautilus-cli -- live validate --config config/live.toml`:
  expected blocker; exited with code 1 and reported that `live validate` is
  defined but not implemented yet.
- `scripts/ai/verify_full.sh`: passed. Full mode ran fast checks, clippy,
  workspace Rust tests, isolated log-global tests, golden trace validation, and
  Rust docs.

## Tests Added or Updated

Updated `crates/cli/src/opt.rs` parser tests to cover:

- top-level help listing `backtest`, `sandbox`, and `live`;
- `sandbox` help listing `validate` and `run`;
- `sandbox validate` config parsing;
- `sandbox run` config, run-id, and output parsing;
- `live` help listing `validate` and `run`;
- `live validate` config parsing;
- `live run` config, run-id, and output parsing.

## Behavior Impact

The CLI now exposes stable Rust-first `sandbox` and `live` command surfaces.
Help and argument parsing work locally. Runtime execution still stops before
starting a node and returns an explicit not-implemented blocker, which prevents
accidental live or sandbox execution through an incomplete product path.

## Public API Impact

This PR adds public CLI subcommands:

```text
sandbox validate
sandbox run
live validate
live run
```

The new commands are intentionally non-executing until the runtime wiring task
connects them to validated sandbox/live node flows.

## Migration Note Status

No migration note is required for existing users because no existing CLI command
changed behavior. The new commands are additive and currently blocker-only.

## Rollback Plan

- Remove the `sandbox` and `live` command definitions from `crates/cli/src/opt.rs`.
- Remove `run_sandbox_command` and `run_live_command` routing from
  `crates/cli/src/lib.rs`.
- Remove this evidence file.
- Restore `.agentflow/state/task_status.json` and `.agentflow/leases/RPROD-006.json`
  to the previous task state if the task is abandoned.
