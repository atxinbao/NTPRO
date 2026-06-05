# NCTL-001 - Register GitHub audit issues and v0.2 backlog

Milestone: v0.2.0 Control
Priority: P0
Default role: Control & Scope
Risk: low

## Goal

Mirror completed GitHub audit issues and the next local v0.2/audit backlog into
`.agentflow` and repository documentation before executing more work.

## Scope

- Add local task records for closed GitHub audit issues `#155` through `#161`.
- Add local `NAUDIT-*` follow-up tasks from the latest read-only audit.
- Add `NQA-001` for v0.2 QA closeout and readiness reporting.
- Add a human-readable GitHub issue register.
- Add a structured `.agentflow` GitHub issue mirror.

## Likely files

- `.agentflow/state/task_status.json`
- `.agentflow/state/github_issues.json`
- `docs/rust-cutover/github_issue_register.md`
- `docs/rust-cutover/tasks/GH-*.md`
- `docs/rust-cutover/tasks/NAUDIT-*.md`
- `docs/rust-cutover/tasks/NQA-001.md`
- `docs/rust-cutover/evidence/NCTL-001.md`

## Non-goals

- Do not execute the newly registered audit tasks.
- Do not create GitHub issues.
- Do not modify runtime code.
- Do not change CLI behavior.
- Do not create release tags or GitHub Releases.

## Dependencies

- none

## Acceptance criteria

- `scripts/ai/validate_agentflow_roles.py` passes.
- Closed GitHub issues have local task records and evidence links.
- Future audit items are visible as `TODO` tasks.
- v0.2 QA closeout is visible as `NQA-001`.

## Required commands

```bash
git diff --check
scripts/ai/validate_agentflow_roles.py
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NCTL-001.md`.
