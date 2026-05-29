# RCTL-009 - Codify PR plain Chinese summary and medium-risk automation policy

Milestone: R0 Control
Priority: P0
Default role: Control/Workflow

## Goal

Codify that every PR body includes a plain Chinese summary and align task risk metadata so unattended automation can continue through medium-risk tasks.

## Scope

- Update agent and task execution rules for plain Chinese PR summaries.
- Update PR template fields.
- Normalize current task risk metadata to medium.
- Keep local auto-dispatch from depending on a fast-forwardable local `main`.

## Likely files

- `AGENTS.md`
- `.github/pull_request_template.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/`
- `docs/rust-cutover/`
- `scripts/control/`

## Non-goals

- Do not modify runtime, adapter, Python, PyO3, Cython, or Cargo code.
- Do not change trading behavior.
- Do not bypass explicit removal, release, or gate prefixes.

## Dependencies

- `RBTL-001`

## Acceptance criteria

- PR template and execution rules require a plain Chinese summary.
- Current task `risk_level` metadata is normalized to `medium`.
- Auto-dispatch can create branches from `origin/main` even when local `main` diverges.
- RBTL-001 is closed as merged in local control state.

## Required commands

```bash
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RCTL-009.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RCTL-009.md` with:

- task ID;
- plain Chinese summary;
- files changed;
- commands run;
- command results;
- behavior impact;
- public API impact;
- rollback plan.
