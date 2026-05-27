# RCTL-005 Evidence

Date: 2026-05-27 11:22:22 CST
Executor: Codex
Task ID: RCTL-005
Branch: `ai/RCTL-005-scope-decision-log`

## Summary

Created the NTPRO scope decision log format and installed initial control-plane
scope decisions. The log now records the required owner decision format and the
Python/PyO3/Cython removal gate. This task also synced task metadata so
RCTL-004 is `DONE` and RCTL-005 is `RUNNING` in `.agentflow/state/task_status.json`.

## Files Changed

- Created `.agentflow/leases/RCTL-005.json`.
- Updated `.agentflow/state/task_status.json`.
- Updated `docs/rust-cutover/scope/SCOPE_DECISIONS.md`.
- Created `docs/rust-cutover/evidence/RCTL-005.md`.

## Commands Run

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RCTL-005.md
sed -n '1,220p' docs/rust-cutover/AGENT_ROLES.md
sed -n '1,180p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,180p' docs/rust-cutover/scope/SCOPE_DECISIONS.md
python3 scripts/ai/lease.py claim RCTL-005 --branch ai/RCTL-005-scope-decision-log --agent-id Codex --path docs/rust-cutover/scope/SCOPE_DECISIONS.md --path docs/rust-cutover/evidence/RCTL-005.md --path .agentflow/state/task_status.json --path .agentflow/leases/RCTL-005.json
scripts/ai/validate_agentflow_roles.py
ruby -rjson -e 'JSON.parse(File.read(".agentflow/state/task_status.json")); puts "ok task_status"'
ruby -e 'text=File.read("docs/rust-cutover/scope/SCOPE_DECISIONS.md"); %w[SD-000 SD-001 Python/PyO3/Cython release_gatekeeper].each { |token| abort("missing #{token}") unless text.include?(token) }; puts "ok scope decisions"'
git diff --check
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/lease.py release RCTL-005 --status PR_READY
```

## Command Results

- RCTL-005 task, role contract, execution protocol, and previous scope decision
  log reads completed.
- Lease was claimed at `.agentflow/leases/RCTL-005.json` and released as `PR_READY`.
- `docs/rust-cutover/scope/SCOPE_DECISIONS.md` now includes:
  - decision states;
  - decision types;
  - required owner decision format;
  - `SD-000` for installing the log format;
  - `SD-001` for gating Python/PyO3/Cython removal.
- `.agentflow/state/task_status.json` was updated:
  - `RCTL-004` is `DONE`;
  - `RCTL-005` is `RUNNING`.
- `validate_agentflow_roles.py`: passed.
- JSON parse for `.agentflow/state/task_status.json`: passed.
- Scope decision token check: passed.
- `git diff --check`: passed.
- `verify_fast.sh`: passed with `PATH="/opt/homebrew/opt/rustup/bin:$PATH"`.
  - `cargo fmt --check` passed.
  - Default fast mode skipped optional cargo check and clippy by design.

## Tests Added or Updated

No runtime tests were added or updated. RCTL-005 is a control-plane scope
decision log task.

## Behavior Impact

No runtime behavior impact. No trading semantics, public APIs, adapters, CLI
behavior, precision behavior, Python/PyO3/Cython product surfaces, or Cargo
workspace configuration were changed.

## Public API Impact

None.

## Migration Note Status

Not required.

## Rollback Plan

- Revert `docs/rust-cutover/scope/SCOPE_DECISIONS.md` to the previous empty log.
- Remove `docs/rust-cutover/evidence/RCTL-005.md`.
- Revert `.agentflow/state/task_status.json` to the previous task-state snapshot.
- Release or remove `.agentflow/leases/RCTL-005.json` if abandoning this branch.
