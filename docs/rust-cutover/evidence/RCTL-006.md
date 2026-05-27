# RCTL-006 Evidence

Date: 2026-05-27 12:17:31 CST
Executor: Codex
Task ID: RCTL-006
Branch: `ai/RCTL-006-pr-issue-templates`

## Summary

Installed NTPRO Rust cutover PR and issue templates. The PR template now
requires task, role, lease, path scope, evidence, impact, and rollback details.
The Rust cutover issue form now captures owner/review roles, risk level,
prohibited paths, dependencies, evidence, and gate confirmations.

## Files Changed

- Updated `.github/pull_request_template.md`.
- Updated `.github/ISSUE_TEMPLATE/rust_cutover_task.yml`.
- Updated `docs/rust-cutover/AGENT_ROLES.md`.
- Updated `.agentflow/policies/path_scope.yaml`.
- Created `.agentflow/leases/RCTL-006.json`.
- Updated `.agentflow/state/task_status.json`.
- Created `docs/rust-cutover/evidence/RCTL-006.md`.

## Commands Run

```bash
node --input-type=module <manual Shrimp MCP isolation smoke>
git status --short --branch
sed -n '1,220p' docs/rust-cutover/tasks/RCTL-006.md
sed -n '1,220p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,220p' docs/rust-cutover/AGENT_ROLES.md
python3 scripts/ai/lease.py claim RCTL-006 --branch ai/RCTL-006-pr-issue-templates --agent-id Codex --path .github/pull_request_template.md --path .github/ISSUE_TEMPLATE/rust_cutover_task.yml --path docs/rust-cutover/evidence/RCTL-006.md --path .agentflow/state/task_status.json --path .agentflow/leases/RCTL-006.json
python3 scripts/ai/lease.py claim RCTL-006 --force --branch ai/RCTL-006-pr-issue-templates --agent-id Codex --path .github/pull_request_template.md --path .github/ISSUE_TEMPLATE/rust_cutover_task.yml --path docs/rust-cutover/AGENT_ROLES.md --path .agentflow/policies/path_scope.yaml --path docs/rust-cutover/evidence/RCTL-006.md --path .agentflow/state/task_status.json --path .agentflow/leases/RCTL-006.json
ruby -rjson -e 'JSON.parse(File.read(".agentflow/state/task_status.json")); puts "ok task_status"'
ruby -ryaml -e 'ARGV.each{|p| YAML.load_file(p); puts "ok #{p}"}' .agentflow/roles.yaml .agentflow/policies/gates.yaml .agentflow/policies/path_scope.yaml .github/ISSUE_TEMPLATE/rust_cutover_task.yml
ruby -e 'checks={".github/pull_request_template.md"=>["Task ID","Owner role","Review role","Risk level","Evidence","Rollback Plan","Review Gate","Python, PyO3, or Cython"],".github/ISSUE_TEMPLATE/rust_cutover_task.yml"=>["owner_role","review_role","risk_level","prohibited_paths","evidence","gates","RCTL-006"]}; checks.each{|path,tokens| text=File.read(path); tokens.each{|token| abort("missing #{token} in #{path}") unless text.include?(token)}; puts "ok #{path}"}'
scripts/ai/validate_agentflow_roles.py
git diff --check
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/lease.py release RCTL-006 --status PR_READY
```

## Command Results

- Manual Shrimp MCP isolation smoke read the NTPRO project queue from
  `/Users/mac/.codex/shrimp-data/NTPRO/tasks.json` and showed:
  - `pending`: 94;
  - `in_progress`: 1;
  - `completed`: 5;
  - active task: `RCTL-006 Install PR and issue templates`.
- Working tree was clean before the RCTL-006 branch.
- RCTL-006 task, execution protocol, and role protocol were read.
- RCTL-006 lease was claimed at `.agentflow/leases/RCTL-006.json`.
- The lease was updated to include Control Agent template path-scope changes.
- `.agentflow/state/task_status.json` was updated:
  - `RCTL-005` is `DONE`;
  - `RCTL-006` is `RUNNING`.
- JSON parse passed for `.agentflow/state/task_status.json`.
- YAML parse passed for role, gate, path-scope, and Rust cutover issue template
  files.
- Template token checks passed for the PR template and Rust cutover issue
  template. An earlier token-check command used an incorrect escaped comma
  string and failed before the corrected command passed.
- `validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.
- `verify_fast.sh`: passed with `PATH="/opt/homebrew/opt/rustup/bin:$PATH"`.
  - `cargo fmt --check` passed.
  - Default fast mode skipped optional cargo check and clippy by design.
- RCTL-006 lease was released as `PR_READY`.

## Tests Added or Updated

No runtime tests were added or updated. RCTL-006 is a control-plane template
task.

## Behavior Impact

No runtime behavior impact. No trading semantics, public APIs, adapters, CLI
behavior, precision behavior, Python/PyO3/Cython product surfaces, or Cargo
workspace configuration were changed.

## Public API Impact

None.

## Migration Note Status

Not required.

## Rollback Plan

- Revert `.github/pull_request_template.md`.
- Revert `.github/ISSUE_TEMPLATE/rust_cutover_task.yml`.
- Remove `docs/rust-cutover/evidence/RCTL-006.md`.
- Revert `.agentflow/state/task_status.json` to the previous task-state
  snapshot.
- Release or remove `.agentflow/leases/RCTL-006.json` if abandoning this branch.
