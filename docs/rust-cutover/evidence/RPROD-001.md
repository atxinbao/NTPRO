# RPROD-001 Evidence

Date: 2026-05-28 00:39:11 CST
Executor: Codex heartbeat `ntpro-auto-dispatch-loop`
Task ID: RPROD-001
Branch: `ai/RPROD-001-define-rust-cli-command-contract`

## Summary

Defined the NTPRO Rust CLI command contract for backtest, sandbox, live, data,
database, and feature-gated blockchain workflows. The contract records the
current CLI baseline, identifies owner-visible product blockers, and maps the
required commands to later RPROD tasks.

## Files Changed

- Created `docs/rust-cutover/product/RUST_CLI_CONTRACT.md`.
- Created `docs/rust-cutover/evidence/RPROD-001.md`.
- Updated `.agentflow/state/task_status.json`.
- Updated `.agentflow/leases/RPROD-001.json`.

## Commands Run

```bash
git status --short --branch
sed -n '1,220p' AGENTS.md
sed -n '1,220p' docs/rust-cutover/tasks/RPROD-001.md
sed -n '1,220p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,220p' docs/rust-cutover/AGENT_ROLES.md
sed -n '1,260p' docs/rust-cutover/inventory/RUST_PRODUCT_SURFACE.md
sed -n '1,260p' crates/cli/src/opt.rs
sed -n '1,220p' crates/cli/src/bin/cli.rs
sed -n '1,220p' crates/cli/Cargo.toml
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- database --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- backtest --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- live --help
python3 scripts/ai/lease.py claim RPROD-001 --force --branch ai/RPROD-001-define-rust-cli-command-contract --agent-id Codex --path docs/rust-cutover/product/RUST_CLI_CONTRACT.md --path docs/rust-cutover/evidence/RPROD-001.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-001.json
ruby -rjson -e 'JSON.parse(File.read(".agentflow/state/task_status.json")); JSON.parse(File.read(".agentflow/leases/RPROD-001.json")); puts "ok json"'
git diff --check
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/lease.py release RPROD-001 --status PR_READY
```

## Command Results

- Auto-dispatch had already created branch
  `ai/RPROD-001-define-rust-cli-command-contract`.
- Shrimp showed one in-progress task:
  `RPROD-001 Define Rust CLI command contract`.
- RPROD-001 task file, task execution protocol, agent roles, Rust product
  inventory, CLI source, and relevant README files were read.
- `cargo run -q -p nautilus-cli -- --help` passed and showed only the default
  `database` top-level command.
- `cargo run -q -p nautilus-cli -- database --help` passed and showed `init`
  and `drop`.
- `cargo run -q -p nautilus-cli -- backtest --help` failed with exit code 2:
  `unrecognized subcommand 'backtest'`.
- `cargo run -q -p nautilus-cli -- live --help` failed with exit code 2:
  `unrecognized subcommand 'live'`.
- The backtest/live failures are expected evidence for this contract task and
  are recorded as product-surface blockers for later RPROD implementation
  tasks.

## Validation Results

- `.agentflow/state/task_status.json` and
  `.agentflow/leases/RPROD-001.json` JSON parse passed.
- `scripts/ai/validate_agentflow_roles.py` passed:
  `agentflow role protocol validation passed`.
- `git diff --check` passed.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh`
  passed:
  - toolchain check completed;
  - `cargo fmt --check` completed;
  - cargo check was skipped by fast-mode default;
  - clippy was skipped by fast-mode default.
- RPROD-001 lease was released as `PR_READY`.

## Tests Added or Updated

No runtime tests were added or updated. RPROD-001 is a product contract and
evidence task.

## Behavior Impact

No runtime behavior impact. No trading semantics, adapters, public APIs,
precision behavior, Python/PyO3/Cython product surfaces, Cargo workspace
configuration, or build features were changed.

## Public API Impact

None. This task documents the intended Rust CLI product contract but does not
change the current CLI implementation.

## Migration Note Status

Not required.

## Rollback Plan

- Remove `docs/rust-cutover/product/RUST_CLI_CONTRACT.md`.
- Remove `docs/rust-cutover/evidence/RPROD-001.md`.
- Revert `.agentflow/state/task_status.json` to the previous task-state
  snapshot if abandoning the branch.
- Release or remove `.agentflow/leases/RPROD-001.json` if abandoning the branch.
