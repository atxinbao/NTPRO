# RCTL-008 Evidence

Date: 2026-05-27 23:11:58 CST
Executor: Codex
Task ID: RCTL-008
Branch: `ai/RCTL-008-pr-autodispatch`

## Summary

Validated the control-plane task state and installed local PR auto-dispatch
support. The new scripts close merged PRs only after the required GitHub check
is green, then dispatch the next eligible Shrimp task from the NTPRO project
queue while preserving local lease and path-scope rules.

## Files Changed

- Created `scripts/control/close_merged_pr.py`.
- Created `scripts/control/dispatch_next.py`.
- Created `docs/rust-cutover/automation/PR_AUTODISPATCH.md`.
- Created `.agentflow/leases/RCTL-008.json`.
- Updated `.agentflow/state/task_status.json`.
- Created `docs/rust-cutover/evidence/RCTL-008.md`.

## Commands Run

```bash
git status --short --branch
sed -n '1,220p' docs/rust-cutover/tasks/RCTL-008.md
python3 scripts/ai/lease.py claim RCTL-008 --branch ai/RCTL-008-pr-autodispatch --agent-id Codex --path scripts/control/close_merged_pr.py --path scripts/control/dispatch_next.py --path docs/rust-cutover/automation/PR_AUTODISPATCH.md --path docs/rust-cutover/evidence/RCTL-008.md --path .agentflow/state/task_status.json --path .agentflow/leases/RCTL-008.json
python3 -m py_compile scripts/control/close_merged_pr.py scripts/control/dispatch_next.py
python3 scripts/control/close_merged_pr.py --pr 11 --task-id RCTL-007 --dry-run
python3 scripts/control/dispatch_next.py --dry-run
python3 scripts/control/dispatch_next.py --dry-run --shrimp-tasks <temporary-completed-RCTL-008-tasks.json>
scripts/ai/validate_agentflow_roles.py
git diff --check
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/lease.py release RCTL-008 --status PR_READY
```

## Command Results

- Initial working tree was clean on `main`.
- RCTL-008 task file was read.
- RCTL-008 lease was claimed at `.agentflow/leases/RCTL-008.json`.
- Shrimp RCTL-008 was marked `in_progress` in the isolated NTPRO queue after
  backing up `/Users/mac/.codex/shrimp-data/NTPRO/tasks.json`.
- `.agentflow/state/task_status.json` was updated:
  - `RCTL-007` is `DONE`;
  - `RCTL-008` is `RUNNING`.
- `py_compile` passed for both control scripts.
- `close_merged_pr.py --dry-run` against PR #11 returned
  `status: already_closed`, confirming repeated heartbeat checks are idempotent
  once the Shrimp task is already completed.
- `dispatch_next.py --dry-run` against the live Shrimp file returned
  `status: waiting` because `RCTL-008` is currently `in_progress`; this is the
  intended no-race behavior.
- `dispatch_next.py --dry-run` against a temporary Shrimp copy with RCTL-008
  marked completed selected:
  - task: `RPROD-001`;
  - branch: `ai/RPROD-001-define-rust-cli-command-contract`;
  - risk: `medium`.

## Validation Results

- `python3 -m py_compile scripts/control/close_merged_pr.py
  scripts/control/dispatch_next.py` passed.
- `.agentflow/state/task_status.json` and `.agentflow/leases/RCTL-008.json`
  JSON parse passed.
- `scripts/ai/validate_agentflow_roles.py` passed:
  `agentflow role protocol validation passed`.
- `git diff --check` passed.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh`
  passed:
  - toolchain check completed;
  - `cargo fmt --check` completed;
  - cargo check was skipped by fast-mode default;
  - clippy was skipped by fast-mode default.
- RCTL-008 lease was released as `PR_READY`.

## Tests Added or Updated

No runtime tests were added or updated. The task adds local control scripts and
documentation with dry-run validation.

## Behavior Impact

No runtime behavior impact. No trading semantics, adapters, public APIs, CLI
behavior, precision behavior, Python/PyO3/Cython product surfaces, Cargo
workspace configuration, or build features were changed.

## Public API Impact

None.

## Migration Note Status

Not required.

## Rollback Plan

- Revert `scripts/control/close_merged_pr.py`.
- Revert `scripts/control/dispatch_next.py`.
- Revert `docs/rust-cutover/automation/PR_AUTODISPATCH.md`.
- Remove `docs/rust-cutover/evidence/RCTL-008.md`.
- Revert `.agentflow/state/task_status.json` to the previous task-state
  snapshot.
- Release or remove `.agentflow/leases/RCTL-008.json` if abandoning this branch.
