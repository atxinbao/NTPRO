# Task Execution Protocol

Date: 2026-07-16
Executor: Codex

## Active Control Plane

GitHub issues, labels, milestones, branches, pull requests, and hosted checks
are the only active task-control authority. Local AgentFlow/Shrimp state and
lease files are retired and must not be used to decide task status.

## Before Work

1. Read `AGENTS.md`, the GitHub issue, and the task file.
2. Confirm the issue is open, its dependencies are closed, and `agent-ready`
   is present.
3. Resolve owner role, review role, risk, allowed paths, prohibited paths, and
   required evidence from the issue and repository policy.
4. Inspect open pull requests for overlapping work.
5. Fetch `origin/main` and create `codex/<task-id>-<slug>` from that commit.
6. Keep one issue, one branch, and one pull request.

## Role Protocol

- Every task declares an owner role and a different review role.
- Owner role may implement and prepare evidence but must not approve its own
  work.
- `BLOCKED` and `QA_PASSED` are not `DONE`.
- `DONE` requires merged PR evidence and issue closure unless the task is
  explicitly local-only.
- Work above medium risk stops at `REVIEW_REQUIRED` before merge and must not
  enable auto-merge.
- Critical removal or release work requires explicit gatekeeper approval.

## Risk Protocol

- Low: docs, examples, task metadata, and inventory-only changes.
- Medium: Rust CLI, non-runtime governance tools, adapter mock tests, and
  scoped CI changes.
- High: workspace restructuring, runtime logic, adapter behavior, persistence
  formats, and feature behavior.
- Critical: product/runtime surface removal, release contract changes, release
  tags, task graph gate changes, and production adapter behavior.

An approved tooling-closeout issue may assign medium risk to deletion of a
proven unreachable helper. That exception does not authorize product/runtime
surface removal.

## During Work

- Keep diffs scoped and do not modify unrelated files.
- Add tests and evidence for behavior changes.
- Preserve the v0.32.0 frozen backend baseline.
- Do not inherit submit, mutation, adapter send, live exchange, retry,
  remediation, recovery, or trading-control capability.

## After Work

1. Run targeted validation and `scripts/ai/verify_fast.sh` when feasible.
2. Write evidence under `docs/rust-cutover/evidence/<task-id>.md`.
3. Fill the PR template, including plain Chinese summary, impact, validation,
   migration status, and rollback plan.
4. Put `Closes #<ISSUE_NUMBER>` in the PR body.
5. Wait for required hosted checks and complete review before merge.
6. For work above medium risk, write the final Chinese handoff and stop at
   `REVIEW_REQUIRED`.
7. After merge, verify the issue closed and move `agent-ready` to the next
   dependency-unblocked issue.

## Blockers

Record the blocker, commands attempted, logs, proposed next action, and whether
an explicit scope or owner decision is required. Do not convert a blocker into
completion evidence.
