# Scope Decisions

Date: 2026-05-27
Executor: Codex
Task ID: RCTL-005

## Purpose

This is the append-only scope decision log for the NTPRO Rust-first cutover.
It records approved deviations from v1 behavior, product route decisions,
Python/PyO3/Cython removal decisions, adapter support decisions, public API
changes, and release gate decisions.

Scope decisions are control-plane artifacts. They do not change runtime
behavior by themselves.

## Decision States

- `PROPOSED`: candidate decision under discussion.
- `APPROVED`: accepted and active.
- `REJECTED`: explicitly rejected.
- `SUPERSEDED`: replaced by a later decision.
- `DEFERRED`: postponed and not executable.
- `BLOCKED`: cannot proceed until the blocker is resolved.

## Decision Types

- `route`: product route, such as Rust-first compatibility vs Rust-only fork.
- `surface`: supported product surface or public API boundary.
- `removal`: Python, PyO3, Cython, adapter, or build-path removal.
- `adapter`: adapter support, deferral, fixture, or parity policy.
- `migration`: user migration behavior or breaking-change policy.
- `release`: release gate, checklist, or signoff policy.
- `control`: workflow, state, task graph, or gate policy.

## Required Format

Every new decision must include:

```text
ID:
State:
Type:
Date:
Owner role:
Review role:
Impacted tasks:
Decision:
Rationale:
Evidence required:
Rollback / supersession:
```

Rules:

- Do not delete historical decisions.
- Supersede decisions by adding a new entry and marking the old entry
  `SUPERSEDED`.
- `BLOCKED` and `DEFERRED` are not equivalent to approval.
- Removal decisions require Verification & Release Gatekeeper review.
- Critical removals require all gates listed in `.agentflow/policies/gates.yaml`.

## Decision Index

| ID | State | Type | Decision | Impacted tasks | Owner role | Review role | Date |
|---|---|---|---|---|---|---|---|
| SD-000 | APPROVED | control | Install this append-only scope decision log format. | RCTL-005 | control_scope_agent | verification_release_gatekeeper | 2026-05-27 |
| SD-001 | APPROVED | removal | Python/PyO3/Cython removal is gated and cannot begin until Rust product surface, runtime smoke, adapter decisions, QA, and release gate evidence are complete. | RREM-*, RREL-*, RPROD-*, RCORE-*, RADP-* | control_scope_agent | verification_release_gatekeeper | 2026-05-27 |

## SD-000 - Scope Decision Log Format

ID: `SD-000`

State: `APPROVED`

Type: `control`

Date: 2026-05-27

Owner role: `control_scope_agent`

Review role: `verification_release_gatekeeper`

Impacted tasks:

- `RCTL-005`

Decision:

Install this file as the canonical append-only scope decision log for NTPRO.
Every future route, removal, adapter support, migration, and release gate
decision must be recorded here before dependent work is marked executable.

Rationale:

NTPRO contains Rust crates, Python package surfaces, PyO3 bindings, Cython
build dependencies, and many adapters. A durable decision log prevents agents
from treating implicit assumptions as approved scope.

Evidence required:

- This file exists.
- RCTL-005 evidence records the creation and validation commands.

Rollback / supersession:

- Supersede with a later `control` decision if the decision schema changes.

## SD-001 - Python, PyO3, and Cython Removal Gate

ID: `SD-001`

State: `APPROVED`

Type: `removal`

Date: 2026-05-27

Owner role: `control_scope_agent`

Review role: `verification_release_gatekeeper`

Impacted tasks:

- `RREM-*`
- `RREL-*`
- `RPROD-*`
- `RCORE-*`
- `RADP-*`

Decision:

Python, PyO3, and Cython removal remains a gated Rust-only cutover path. No
agent may delete or disable `python/**`, `nautilus_trader/**`, `crates/pyo3/**`,
`build.py`, `pyproject.toml`, Cython files, or related active product build
paths until the release gatekeeper confirms the required gates are complete.

Required gates:

- Rust product surface ready.
- Runtime smoke passed.
- Adapter decisions recorded.
- QA gate passed.
- Release gatekeeper approved.
- Rust-only route or removal task explicitly approved.

Rationale:

The repository is currently a Rust-first cutover workspace with legacy Python,
PyO3, and Cython surfaces still present. Premature removal would break product
surface, parity, packaging, or adapter workflows before replacement evidence is
available.

Evidence required:

- Rust CLI/API/example usability evidence.
- Runtime smoke evidence.
- Adapter inventory and support decisions.
- QA and release gate evidence.
- Residual Python/PyO3/Cython report before removal work begins.

Rollback / supersession:

- Supersede only with a later `removal` or `route` decision reviewed by the
  Verification & Release Gatekeeper.
