# RPROD-002 Evidence

Date: 2026-05-28 00:48:29 CST
Executor: Codex heartbeat `ntpro-auto-dispatch-loop`
Task ID: RPROD-002
Branch: `ai/RPROD-002-add-rust-cli-help-and-version-smoke`

## Summary

Added a local Rust CLI smoke command for the currently supported `nautilus-cli`
surface. The smoke verifies top-level help, version output, and database help
without requiring Python, PyO3, or Cython.

## Files Changed

- Created `scripts/ai/verify_cli_help.sh`.
- Created `docs/rust-cutover/evidence/RPROD-002.md`.
- Updated `.agentflow/state/task_status.json`.
- Updated `.agentflow/leases/RPROD-002.json`.

## Commands Run

```bash
git status --short --branch
sed -n '1,220p' docs/rust-cutover/tasks/RPROD-002.md
python3 scripts/ai/lease.py claim RPROD-002 --force --branch ai/RPROD-002-add-rust-cli-help-and-version-smoke --agent-id Codex --path scripts/ai/verify_cli_help.sh --path docs/rust-cutover/evidence/RPROD-002.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-002.json
scripts/ai/verify_cli_help.sh
ruby -rjson -e 'JSON.parse(File.read(".agentflow/state/task_status.json")); JSON.parse(File.read(".agentflow/leases/RPROD-002.json")); puts "ok json"'
scripts/ai/validate_agentflow_roles.py
git diff --check
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/lease.py release RPROD-002 --status PR_READY
```

## Command Results

- Auto-dispatch created branch
  `ai/RPROD-002-add-rust-cli-help-and-version-smoke`.
- Shrimp showed one in-progress task:
  `RPROD-002 Add Rust CLI help and version smoke`.
- RPROD-002 task file was read.
- RPROD-002 lease was expanded to include the CLI smoke script and evidence.
- `scripts/ai/verify_cli_help.sh` passed and ran:
  - `cargo run -q -p nautilus-cli -- --help`;
  - `cargo run -q -p nautilus-cli -- --version`;
  - `cargo run -q -p nautilus-cli -- database --help`.
- Observed version output: `nautilus-cli 0.58.0`.

## Validation Results

- `.agentflow/state/task_status.json` and
  `.agentflow/leases/RPROD-002.json` JSON parse passed.
- `scripts/ai/validate_agentflow_roles.py` passed:
  `agentflow role protocol validation passed`.
- `git diff --check` passed.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh`
  passed:
  - toolchain check completed;
  - `cargo fmt --check` completed;
  - cargo check was skipped by fast-mode default;
  - clippy was skipped by fast-mode default.
- RPROD-002 lease was released as `PR_READY`.

## Tests Added or Updated

Added `scripts/ai/verify_cli_help.sh` as a local smoke command for current Rust
CLI help/version output.

## Behavior Impact

No runtime behavior impact. No trading semantics, adapters, public APIs,
precision behavior, Python/PyO3/Cython product surfaces, Cargo workspace
configuration, or build features were changed.

## Public API Impact

None. The script validates current CLI output but does not change CLI behavior.

## Migration Note Status

Not required.

## Rollback Plan

- Remove `scripts/ai/verify_cli_help.sh`.
- Remove `docs/rust-cutover/evidence/RPROD-002.md`.
- Revert `.agentflow/state/task_status.json` to the previous task-state
  snapshot if abandoning the branch.
- Release or remove `.agentflow/leases/RPROD-002.json` if abandoning the branch.
