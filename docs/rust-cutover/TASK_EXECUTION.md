# Task Execution Protocol

## Before work

1. Read `AGENTS.md`.
2. Read the task file.
3. Read `docs/rust-cutover/AGENT_ROLES.md`.
4. Resolve `owner_role`, `review_role`, `risk_level`, allowed paths, prohibited
   paths, and required evidence from `.agentflow/state/task_status.json`,
   `.agentflow/roles.yaml`, and `.agentflow/policies/path_scope.yaml`.
5. Check `.agentflow/leases/` for path conflicts.
6. Claim a lease.
7. Create a branch named `ai/<task-id>-<slug>`.

## Role protocol

- Every task must have one `owner_role` and one different `review_role`.
- Owner role may execute implementation and prepare evidence.
- Review role validates evidence and gate requirements.
- Owner role must not approve its own task.
- `BLOCKED` is not `DONE`.
- `QA_PASSED` is not `DONE`.
- `DONE` requires QA evidence, review evidence, and merged PR evidence unless the
  task explicitly documents that it is local-only.
- High-risk work must stop at `REVIEW_REQUIRED` before merge.
- High-risk work must not enable auto-merge.
- Critical removal or release work requires explicit release gatekeeper approval.

## Risk protocol

- Low risk: docs, examples, task metadata, non-runtime scripts, inventory docs.
- Medium risk: Rust CLI, runtime-facing examples, adapter mock tests, CI, Cargo
  feature cleanup.
- High risk: workspace restructuring, runtime logic, adapter behavior,
  persistence format, feature flag behavior.
- Critical risk: deleting Python, PyO3, Cython, `build.py`, `pyproject.toml`,
  release contract changes, task graph gate changes, release tags, and
  production adapter behavior changes.

## During work

- Keep diffs small.
- Prefer tests before code changes.
- Do not modify unrelated files.
- Add docs when public behavior changes.
- Stay inside the task path scope unless a scope decision explicitly allows a
  wider change.

## After work

1. Run targeted commands.
2. Run `scripts/ai/verify_fast.sh` if feasible.
3. Write evidence under `docs/rust-cutover/evidence/<task-id>.md`.
4. Run `scripts/ai/validate_agentflow_roles.py` for control-plane task metadata
   changes.
5. Fill PR template.
6. For high-risk tasks, write the final handoff in plain Chinese before
   technical details. It must state what changed, what did not change, why the
   task is high risk, validation results, PR link, and gate/review status.
7. Release lease only after PR is ready or task is blocked.

## Blockers

If blocked, write:

- blocker summary;
- commands attempted;
- relevant logs;
- proposed next action;
- whether scope/human decision is needed.
