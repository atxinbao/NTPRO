# RCTL-007 Evidence

Date: 2026-05-27 23:00:44 CST
Executor: Codex
Task ID: RCTL-007
Branch: `ai/RCTL-007-bootstrap-evidence`

## Summary

Updated the NTPRO bootstrap evidence so it reflects the current repository
baseline after the control-plane PRs, the `main` default branch change, GitHub
branch protection, CI smoke cleanup, Shrimp project isolation, and NTPRO-only
remote configuration. This task also creates the dedicated RCTL-007 evidence
file required by the task contract.

## Files Changed

- Updated `docs/rust-cutover/evidence/bootstrap_evidence.md`.
- Created `docs/rust-cutover/evidence/RCTL-007.md`.
- Created `.agentflow/leases/RCTL-007.json`.
- Updated `.agentflow/state/task_status.json`.

## Commands Run

```bash
git status --short --branch
sed -n '1,220p' docs/rust-cutover/tasks/RCTL-007.md
sed -n '1,240p' docs/rust-cutover/CONTRACT.md
sed -n '1,240p' docs/rust-cutover/DEFINITION_OF_DONE.md
sed -n '1,260p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,260p' docs/rust-cutover/AGENT_ROLES.md
jq '.tasks["RCTL-006"], .tasks["RCTL-007"], .tasks["RCTL-008"]' .agentflow/state/task_status.json
python3 scripts/ai/lease.py claim RCTL-007 --branch ai/RCTL-007-bootstrap-evidence --agent-id Codex --path docs/rust-cutover/evidence/bootstrap_evidence.md --path docs/rust-cutover/evidence/RCTL-007.md --path .agentflow/state/task_status.json --path .agentflow/leases/RCTL-007.json
gh pr list -R atxinbao/NTPRO --state merged --limit 20 --json number,title,mergedAt,baseRefName,headRefName,url --jq '.[] | "#\(.number) | \(.title) | base=\(.baseRefName) | head=\(.headRefName) | merged=\(.mergedAt) | \(.url)"'
gh api repos/atxinbao/NTPRO --jq '{default_branch,visibility,allow_auto_merge,delete_branch_on_merge}'
gh api repos/atxinbao/NTPRO/branches/main/protection --jq '{required_status_checks,required_pull_request_reviews,enforce_admins,allow_force_pushes,allow_deletions}'
git rev-parse --short HEAD
git log -1 --pretty='%H%n%ci%n%s'
git remote -v
ruby -rjson -e 'JSON.parse(File.read(".agentflow/state/task_status.json")); puts "ok task_status"'
ruby -ryaml -e 'ARGV.each{|p| YAML.load_file(p); puts "ok #{p}"}' .agentflow/roles.yaml .agentflow/policies/gates.yaml .agentflow/policies/path_scope.yaml
scripts/ai/validate_agentflow_roles.py
git diff --check
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/lease.py release RCTL-007 --status PR_READY
```

## Command Results

- Initial working tree was clean on `main`.
- RCTL-007 task file, cutover contract, Definition of Done, task execution
  protocol, and agent role protocol were read.
- Shrimp `list_tasks` showed the NTPRO queue with 100 tasks and RCTL-007 as the
  next pending task before execution.
- Code-Index was refreshed for `/Users/mac/Documents/NTPRO`.
- RCTL-007 lease was claimed at `.agentflow/leases/RCTL-007.json`.
- GitHub repository metadata confirmed:
  - `default_branch`: `main`;
  - `visibility`: `public`;
  - `allow_auto_merge`: `true`;
  - `delete_branch_on_merge`: `true`.
- `main` branch protection requires strict `smoke`; force pushes and branch
  deletion are disabled.
- Merged PR history through PR #10 was read and summarized in
  `bootstrap_evidence.md`.
- `.agentflow/state/task_status.json` was updated:
  - `RCTL-006` is `DONE`;
  - `RCTL-007` is `RUNNING`.
- Final validation results are recorded below after command execution.

## Validation Results

- `.agentflow/state/task_status.json` JSON parse passed:
  `ok task_status`.
- `.agentflow/roles.yaml`, `.agentflow/policies/gates.yaml`, and
  `.agentflow/policies/path_scope.yaml` YAML parse passed.
- `scripts/ai/validate_agentflow_roles.py` passed:
  `agentflow role protocol validation passed`.
- `git diff --check` passed.
- `PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh`
  passed:
  - toolchain check completed;
  - `cargo fmt --check` completed;
  - cargo check was skipped by fast-mode default;
  - clippy was skipped by fast-mode default.
- RCTL-007 lease was released as `PR_READY`.

## Tests Added or Updated

No runtime tests were added or updated. RCTL-007 is a control-plane evidence
task.

## Behavior Impact

No runtime behavior impact. No trading semantics, adapters, public APIs, CLI
behavior, precision behavior, Python/PyO3/Cython product surfaces, Cargo
workspace configuration, or build features were changed.

## Public API Impact

None.

## Migration Note Status

Not required.

## Rollback Plan

- Revert `docs/rust-cutover/evidence/bootstrap_evidence.md`.
- Remove `docs/rust-cutover/evidence/RCTL-007.md`.
- Revert `.agentflow/state/task_status.json` to the previous task-state
  snapshot.
- Release or remove `.agentflow/leases/RCTL-007.json` if abandoning this branch.
