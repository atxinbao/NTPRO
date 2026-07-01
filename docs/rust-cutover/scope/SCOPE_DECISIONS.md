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
| SD-002 | APPROVED | release | v0.18.0 is owner-approved cancel recovery preview/gate/approval evidence only; actual cancel send remains forbidden. | V180-* | control_scope_agent | verification_release_gatekeeper | 2026-06-26 |
| SD-003 | APPROVED | release | v0.20.0 may enter owner-approved production order lifecycle foundation after v0.19.1 closeout evidence, with strict single-shot owner approval, risk, readback, audit, and no-automation boundaries. | V200-* | control_scope_agent | verification_release_gatekeeper | 2026-06-29 |
| SD-004 | APPROVED | release | v0.21.0 may enter unified read model foundation work after v0.20.1 publication evidence, with read-only/foundation boundaries and no submit expansion. | V210-* | control_scope_agent | verification_release_gatekeeper | 2026-06-30 |
| SD-005 | APPROVED | release | v0.22.0 may enter Trader Terminal workbench work after v0.21.1 publication evidence, with read-only-first and gated-operation boundaries. | V220-* | control_scope_agent | verification_release_gatekeeper | 2026-07-01 |

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

## SD-002 - v0.18.0 Owner-Approved Cancel Recovery Preview Boundary

ID: `SD-002`

State: `APPROVED`

Type: `release`

Date: 2026-06-26

Owner role: `control_scope_agent`

Review role: `verification_release_gatekeeper`

Impacted tasks:

- `V180-*`

Decision:

`v0.18.0` may prepare owner-approved cancel recovery preview, cancel risk gate,
manual owner approval lifecycle, redaction, readback, incident/audit closeout,
Dashboard read-only display, and release gate evidence. It must keep actual
cancel send disabled.

Required boundary:

```text
actual_cancel_send_allowed = false
cancel_attempted = false
automatic_cancel_allowed = false
dashboard_cancel_controls_enabled = false
```

Rationale:

v0.17.0 and v0.17.1 remain evidence-only release tracks. Cancel recovery changes
the system toward active order management, so v0.18.0 must first prove preview,
gate, owner approval, redaction, and audit contracts without sending a cancel
request.

Evidence required:

- `docs/rust-cutover/scope/v0_18_0_owner_approved_cancel_recovery_preview.md`
- V180 task evidence proving the boundary fields remain false.
- Release gates proving no cancel endpoint was attempted.

Rollback / supersession:

- Revert V180 scope docs and evidence to block v0.18.0 cancel recovery work.
- Supersede only with a later release scope decision reviewed by the
  Verification & Release Gatekeeper.

## SD-003 - v0.20.0 Owner-Approved Production Order Lifecycle Foundation

ID: `SD-003`

State: `APPROVED`

Type: `release`

Date: 2026-06-29

Owner role: `control_scope_agent`

Review role: `verification_release_gatekeeper`

Impacted tasks:

- `V200-000`
- `V200-001` through `V200-012`

Decision:

`v0.20.0` may proceed from planning into the owner-approved production order
lifecycle foundation only after the V200-000 PR merges. The allowed capability
is a tightly bounded, single-shot, owner-approved submit/readback/cancel/audit
foundation. The scope does not approve a general production trading platform,
strategy-driven production execution, automatic order placement, bulk orders,
retry/replace/amend/flatten, automatic remediation, multi-account or multi-venue
execution, Dashboard order controls, or MARKET orders without a later explicit
scope decision.

Required boundary:

```text
owner_approval_required = true
single_order_required = true
single_venue_required = true
single_attempt_required = true
pre_submit_risk_gate_required = true
post_submit_readback_required = true
audit_artifact_required = true
automatic_order_placement_allowed = false
strategy_driven_production_execution_allowed = false
bulk_order_allowed = false
retry_replace_amend_flatten_allowed = false
dashboard_order_controls_enabled = false
general_production_trading_platform_claim = false
```

Rationale:

The v0.19.0 line established owner-approved single-shot actual cancel, and the
v0.19.1 closeout line completed release evidence, publication guard, strict
provenance, post-merge review attestation, and standalone gate hardening. That
evidence allows v0.20.0 to start only as a controlled lifecycle foundation, not
as an unrestricted trading surface.

Evidence required:

- `docs/rust-cutover/scope/v0_20_0_owner_approved_production_order_lifecycle_foundation.md`
- V200 task evidence proving the owner approval, risk, readback, audit, no retry,
  no bulk, no automatic execution, and no Dashboard control boundaries.
- Release gates and golden traces before any v0.20 release claim.

Rollback / supersession:

- Revert V200-000 scope docs and evidence to block V200-001 through V200-012.
- Supersede only with a later release scope decision reviewed by the
  Verification & Release Gatekeeper.

## SD-004 - v0.21.0 Unified Read Model Foundation Boundary

ID: `SD-004`

State: `APPROVED`

Type: `release`

Date: 2026-06-30

Owner role: `control_scope_agent`

Review role: `verification_release_gatekeeper`

Impacted tasks:

- `V210-*`

Decision:

`v0.21.0` may start its scoped unified read model foundation after the
`v0.20.1` hardening patch was published and all V201 issues were closed. The
scope is read-only account, position, order, fill, and risk projection work
plus a read-only foundation Dashboard. It must not add submit capability,
Dashboard order controls, retry/replace/amend/flatten, strategy-driven live
trading, or product-grade live trading terminal claims.

Rationale:

The v0.20.1 closeout established release evidence, strict provenance,
single-shot attempt ledger hardening, notional consistency, adapter/source
labels, Dashboard foundation-boundary diagnostics, and a successful hosted
release-tag gate. That evidence is sufficient to begin read-model work, but not
to expand order mutation capability.

Evidence required:

- `docs/rust-cutover/scope/v0_21_0_unified_read_model_foundation.md`
- `docs/rust-cutover/evidence/V210-000.md`
- GitHub issue `#651` closed through merged PR evidence.
- Later V210 tasks must preserve no-new-submit and no-Dashboard-operation
  controls evidence.

Rollback / supersession:

- Revert V210-000 scope and evidence docs to restore blocked V210 wording if
  the v0.20.1 publication evidence is found inaccurate.
- Supersede only with a later release scope decision reviewed by the
  Verification & Release Gatekeeper.

## SD-005 - v0.22.0 Trader Terminal Workbench Boundary

ID: `SD-005`

State: `APPROVED`

Type: `release`

Date: 2026-07-01

Owner role: `control_scope_agent`

Review role: `verification_release_gatekeeper`

Impacted tasks:

- `V220-*`

Decision:

`v0.22.0` may start its scoped Trader Terminal workbench line after the
`v0.21.1` hardening patch was published and all V211 issues were closed. The
scope is a read-only first workbench for account, position, order, fill, risk,
alert, audit, and provenance drill-down views, plus gated manual operation entry
design. Any future real operation entry must require owner approval, risk gate,
and audit gate evidence. The scope must not add ungated submit, cancel, retry,
replace, amend, flatten, strategy-driven live trading, or product-grade live
trading terminal claims.

Rationale:

The v0.21.1 closeout established release evidence, strict provenance,
health-status semantics, executable read-model replay, JSON Schema boundary
hardening, Trader Terminal read-model runtime bridge, v0.22 dependency proof,
and a successful hosted release-tag gate. That evidence is sufficient to begin
workbench work, but not to expand order mutation capability or claim a
production trading terminal.

Evidence required:

- `docs/rust-cutover/scope/v0_22_0_trader_terminal_workbench_scope.md`
- `docs/rust-cutover/evidence/V220-000.md`
- GitHub issue `#683` closed through merged PR evidence.
- Later V220 tasks must preserve read-only first, owner approval gate, risk
  gate, audit gate, no-ungated-operation, and no-product-grade-terminal
  evidence.

Rollback / supersession:

- Revert V220-000 scope and evidence docs to restore blocked V220 wording if
  the v0.21.1 publication evidence is found inaccurate.
- Supersede only with a later release scope decision reviewed by the
  Verification & Release Gatekeeper.
