# PR Auto-Dispatch Retirement

Date: 2026-07-16
Executor: Codex
Status: RETIRED

## Decision

The local Python auto-dispatch, closeout, lease, AgentFlow, and Shrimp mutation
path is retired. GitHub is the only active task control plane.
No local queue or lease file is authoritative.

The removed components were `scripts/control/dispatch_next.py`,
`scripts/control/close_merged_pr.py`, `scripts/ai/lease.py`, and
`scripts/ai/validate_agentflow_roles.py`. Their historical evidence remains
under `docs/rust-cutover/`.

## Supported Intake

List dependency-ready work from GitHub:

```bash
gh issue list --repo atxinbao/NTPRO --state open \
  --label agent-ready --json number,title,labels,milestone
```

Confirm dependencies from the issue body and live issue states, then create the
single task branch from current remote main:

```bash
git fetch --prune origin
git switch -c codex/<TASK_ID>-<SLUG> origin/main
```

## Supported Closeout

Inspect the PR and required checks directly:

```bash
gh pr view <PR_NUMBER> --repo atxinbao/NTPRO \
  --json state,mergedAt,mergeCommit,statusCheckRollup
```

Every PR body must contain `Closes #<ISSUE_NUMBER>`. After merge, verify the
issue is closed and move `agent-ready` to the next dependency-unblocked issue.
Use `jq` only when machine-readable filtering is required.

## Boundaries

- No unattended local queue mutation is supported.
- No GitHub workflow may mutate machine-local task state.
- High and critical risk tasks stop at `REVIEW_REQUIRED`.
- One issue, one branch, and one PR remain mandatory.
- The v0.32.0 backend freeze and forbidden trading capability boundaries remain
  unchanged.
