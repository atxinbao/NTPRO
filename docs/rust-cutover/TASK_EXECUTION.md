# Task Execution Protocol

## Before work

1. Read `AGENTS.md`.
2. Read the task file.
3. Check `.agentflow/leases/` for path conflicts.
4. Claim a lease.
5. Create a branch named `ai/<task-id>-<slug>`.

## During work

- Keep diffs small.
- Prefer tests before code changes.
- Do not modify unrelated files.
- Add docs when public behavior changes.

## After work

1. Run targeted commands.
2. Run `scripts/ai/verify_fast.sh` if feasible.
3. Write evidence under `docs/rust-cutover/evidence/<task-id>.md`.
4. Fill PR template.
5. Release lease only after PR is ready or task is blocked.

## Blockers

If blocked, write:

- blocker summary;
- commands attempted;
- relevant logs;
- proposed next action;
- whether scope/human decision is needed.
