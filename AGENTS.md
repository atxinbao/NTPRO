# AGENTS.md — NautilusTrader Rust Cutover Rules

This repository is undergoing a Rust v2 cutover. Coding agents must follow these rules.

## Mission

Make Rust v2 the primary runtime, expose Python as Rust-only product surface, and remove Cython as a required v2 runtime dependency.

## Read first

Before any change, read:

- `docs/rust-cutover/CONTRACT.md`
- `docs/rust-cutover/DEFINITION_OF_DONE.md`
- `docs/rust-cutover/TASK_EXECUTION.md`
- `docs/rust-cutover/AGENT_ROLES.md`
- the task file under `docs/rust-cutover/tasks/`

## Hard rules

1. One issue, one branch, one PR.
2. No drive-by refactors.
3. No behavior change without tests and evidence.
4. No trading-semantic change without golden trace coverage.
5. No public API change without migration note.
6. No new unsafe Rust without justification.
7. No precision change without high/standard precision tests.
8. No adapter behavior change without raw fixture tests.
9. Do not delete or move Cython files unless the current task explicitly says so; dedicated CY tasks must remove v1 Cython entirely before release.
10. If a command cannot run, document why in evidence.
11. High-risk work must stop at `REVIEW_REQUIRED` before merge and must not
    enable auto-merge.
12. After finishing high-risk work, report in plain Chinese first: what changed,
    what did not change, why it is high risk, validation results, PR link, and
    the gate/review status.

## Preferred commands

Fast check:

```bash
scripts/ai/verify_fast.sh
```

Full check:

```bash
scripts/ai/verify_full.sh
```

Release check:

```bash
scripts/ai/verify_release.sh
```

## PR evidence required

Every PR must include:

- task ID;
- goal;
- files changed;
- commands run;
- test result summary;
- behavior impact;
- public API impact;
- migration note status;
- rollback plan.
