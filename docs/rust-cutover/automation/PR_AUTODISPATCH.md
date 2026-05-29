# PR Auto-Dispatch

Date: 2026-05-27
Executor: Codex

## Purpose

NTPRO can automatically continue from one merged PR to the next eligible task
without using GitHub Actions to mutate local Shrimp state. The automation is
local-first because Shrimp task data and Codex workspace state live on this
machine.

## Components

- `scripts/control/close_merged_pr.py`
  - verifies a PR is merged;
  - verifies the required GitHub check is green;
  - keeps local Shrimp state in sync without requiring local `main` to be
    fast-forwardable;
  - marks the completed task in
    `/Users/mac/.codex/shrimp-data/NTPRO/tasks.json`.
- `scripts/control/dispatch_next.py`
  - requires a clean local worktree;
  - fetches `origin/main` and creates the task branch from that remote base;
  - refuses to run while Shrimp has an `in_progress` task;
  - selects the first pending task whose Shrimp dependencies are completed;
  - blocks removal, release, and gate task prefixes by default;
  - allows medium-risk tasks by default;
  - creates `ai/<task-id>-<slug>`;
  - updates `.agentflow/state/task_status.json` for the new branch;
  - claims a base lease for the task;
  - marks the Shrimp task `in_progress`.

## Local Commands

Close a merged PR after `smoke` is green:

```bash
python3 scripts/control/close_merged_pr.py \
  --repo atxinbao/NTPRO \
  --pr <PR_NUMBER> \
  --task-id <TASK_ID> \
  --required-check smoke
```

Dispatch the next eligible task:

```bash
python3 scripts/control/dispatch_next.py \
  --workspace /Users/mac/Documents/NTPRO \
  --shrimp-tasks /Users/mac/.codex/shrimp-data/NTPRO/tasks.json
```

Dry-run either command before enabling unattended execution:

```bash
python3 scripts/control/close_merged_pr.py --pr <PR_NUMBER> --task-id <TASK_ID> --dry-run
python3 scripts/control/dispatch_next.py --dry-run
```

## Codex Heartbeat Contract

The recurring Codex wake-up should do only this:

1. Inspect the active PR if one is recorded in the thread.
2. If the PR is not merged, report `waiting`.
3. If the PR is merged and `smoke` passed, run `close_merged_pr.py`.
4. If no Shrimp task is `in_progress`, run `dispatch_next.py`.
5. Continue the newly dispatched task only if it is medium risk or below.
6. Stop and report when the selected task requires a manual gate.

The heartbeat must not run removal, release, or explicit gate tasks unattended.
Every PR body opened by the heartbeat must include a plain Chinese summary.

## Boundaries

- Do not run this from GitHub Actions. GitHub runners cannot safely mutate the
  local Shrimp queue under `/Users/mac/.codex/shrimp-data/NTPRO`.
- Do not dispatch `RREM-*`, `RREL-*`, `NREM-*`, `NREL-*`, or `NGATE-*`
  without an explicit manual gate.
- Do not dispatch when the local worktree is dirty.
- Do not dispatch when another Shrimp task is already `in_progress`.
- Do not treat `QA_PASSED` or `BLOCKED` as `DONE`.
