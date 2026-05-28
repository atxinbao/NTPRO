# RPROD-005 Evidence

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-005
Branch: `ai/RPROD-005-define-live-or-sandbox-run-cli-contract`

## Summary

Defined the Rust-first live and sandbox run CLI contract in
`docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md`. The contract records
the required command surface, config shape, lifecycle phases, shutdown behavior,
output contract, failure behavior, implementation gates, and current CLI
blockers.

RPROD-005 is a product contract task. It does not implement the `sandbox` or
`live` CLI commands; RPROD-006 owns that implementation or blocker handoff.

## Files Changed

- Created `docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md`.
- Created `docs/rust-cutover/evidence/RPROD-005.md`.
- Updated `.agentflow/state/task_status.json`.
- Updated `.agentflow/leases/RPROD-005.json`.

## Commands Run

```bash
git status --short --branch
sed -n '1,220p' docs/rust-cutover/tasks/RPROD-005.md
sed -n '1,240p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,220p' docs/rust-cutover/AGENT_ROLES.md
ruby -rjson -e 's=JSON.parse(File.read(".agentflow/state/task_status.json")); puts JSON.pretty_generate(s.fetch("tasks").fetch("RPROD-005"))'
python3 scripts/ai/lease.py claim RPROD-005 --force --branch ai/RPROD-005-define-live-or-sandbox-run-cli-contract --agent-id Codex --path docs/rust-cutover/tasks/RPROD-005.md --path docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md --path docs/rust-cutover/evidence/RPROD-005.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-005.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- sandbox --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -q -p nautilus-cli -- live --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
ruby -rjson -e 'JSON.parse(File.read(".agentflow/state/task_status.json")); JSON.parse(File.read(".agentflow/leases/RPROD-005.json")); puts "ok json"'
scripts/ai/validate_agentflow_roles.py
git diff --check
python3 scripts/ai/lease.py release RPROD-005 --status PR_READY
```

## Command Results

- `git status --short --branch`: confirmed the active branch is
  `ai/RPROD-005-define-live-or-sandbox-run-cli-contract`.
- Task and role docs were read before editing.
- RPROD-005 state lookup confirmed owner role `rust_product_surface_agent`,
  review role `verification_release_gatekeeper`, and risk `medium`.
- `lease.py claim`: expanded the RPROD-005 lease to include the live/sandbox CLI
  contract and evidence files.
- `cargo run -q -p nautilus-cli -- sandbox --help`: expected blocker; exited
  with code 2 because the current CLI does not expose `sandbox`.
- `cargo run -q -p nautilus-cli -- live --help`: expected blocker; exited with
  code 2 because the current CLI does not expose `live`.
- `scripts/ai/verify_fast.sh`: passed. Fast mode ran toolchain and rustfmt
  checks, then skipped optional workspace cargo check and clippy because their
  opt-in environment variables were not set.
- JSON parse: passed with `ok json`.
- `scripts/ai/validate_agentflow_roles.py`: passed:
  `agentflow role protocol validation passed`.
- `git diff --check`: passed with no output.
- `lease.py release`: passed and marked `.agentflow/leases/RPROD-005.json` as
  `PR_READY`.

## Tests Added or Updated

No runtime tests were added. RPROD-005 is a product contract task, not a CLI
implementation task.

## Behavior Impact

No runtime behavior changed. The new contract defines expected Rust-first
`sandbox` and `live` command behavior and explicitly records that both commands
are currently absent from the CLI.

## Public API Impact

No current public API changed. The document defines a future CLI contract:

```text
nautilus sandbox validate --config <path>
nautilus sandbox run --config <path> [--run-id <id>] [--output <dir>]
nautilus live validate --config <path>
nautilus live run --config <path> [--run-id <id>] [--output <dir>]
```

## Migration Note Status

No migration note is required for this task because no existing CLI behavior was
changed.

## Rollback Plan

- Remove `docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md`.
- Remove `docs/rust-cutover/evidence/RPROD-005.md`.
- Restore `.agentflow/state/task_status.json` to the previous task state if the
  task is abandoned.
- Release or remove `.agentflow/leases/RPROD-005.json` if abandoning the branch.
