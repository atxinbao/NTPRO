# NTPRO Agent Roles

Date: 2026-05-27
Executor: Codex

## Purpose

NTPRO is managed as a Rust-first cutover workspace. The project may later move
to Rust-only removal, but Python, PyO3, and Cython deletion is gated until
Rust product surface, runtime, adapter, and QA evidence are complete.

The five roles below define task ownership, review boundaries, path authority,
and gate responsibility. They do not require five always-open Codex sessions.
Each task must declare an owner role, a review role, risk level, allowed paths,
required evidence, and done requirements.

## Roles

### Control & Scope Agent

Purpose: scope control, task graph, leases, gates, state transitions, and scope
decisions.

Owns:

- `RCTL-*`
- future `NCTL-*`, `NSCOPE-*`, `NPLAN-*`

May modify:

- `.agentflow/**`
- `backlog/**`
- `state/**`
- `scripts/control/**`
- `docs/architecture/**`
- `docs/rust-cutover/**`
- `.github/pull_request_template.md`
- `.github/ISSUE_TEMPLATE/**`
- `NTPRO_CONTRACT.md`

May not modify unless explicitly authorized:

- `crates/**`
- `python/**`
- `nautilus_trader/**`
- `crates/pyo3/**`
- `pyproject.toml`
- `build.py`
- `Cargo.toml`
- `Cargo.lock`

Rules:

- May move tasks through control states.
- May record `BLOCKED`.
- Must not mark code tasks `DONE`.
- Must not treat `BLOCKED` as `DONE`.
- Must not change runtime code to satisfy control-plane tasks.

### Rust Product Surface Agent

Purpose: Rust CLI, Rust API ergonomics, Rust examples, Rust documentation, and
Rust-first user entrypoints.

Owns:

- `RPROD-*`
- future `NPROD-*`, `NCLI-*`, `NEXAMPLE-*`, `NDOCS-RUST-*`

May modify:

- `crates/cli/**`
- `crates/**/src/**`
- `examples/**`
- `docs/**`
- `tests/**`
- `README.md`
- `Cargo.toml`
- `Cargo.lock`

May not modify unless explicitly authorized:

- `backlog/**`
- `state/**`
- `.agentflow/policies/**`
- `python/**`
- `nautilus_trader/**`
- `crates/pyo3/**`
- `pyproject.toml`
- `build.py`

Required evidence:

- CLI help output for touched commands.
- Rust example compile evidence when examples change.
- Targeted Cargo test or documented reason for not running.
- No new Python, PyO3, or Cython product dependency.

### Rust Core Runtime Agent

Purpose: Rust runtime internals for backtest, live, trading, data, execution,
risk, portfolio, model, persistence, and system lifecycle.

Owns:

- `RCORE-*`
- `RBTL-*`
- future `NCORE-*`, `NRUNTIME-*`, `NBACKTEST-*`, `NLIVE-*`, `NENGINE-*`

May modify:

- `crates/backtest/**`
- `crates/live/**`
- `crates/trading/**`
- `crates/common/**`
- `crates/core/**`
- `crates/data/**`
- `crates/execution/**`
- `crates/risk/**`
- `crates/portfolio/**`
- `crates/model/**`
- `crates/persistence/**`
- `crates/system/**`
- `tests/**`

May not modify unless explicitly authorized:

- `backlog/**`
- `state/**`
- `.agentflow/policies/**`
- `docs/release/**`
- `python/**`
- `nautilus_trader/**`
- `crates/pyo3/**`

Required evidence:

- Runtime smoke or targeted crate tests.
- Changed crate cargo checks or documented reason for not running.
- No new Python-first runtime dependency report.

### Adapter & Integration Agent

Purpose: venue adapters, data-provider adapters, external integration
boundaries, sandbox, database, DeFi, network, fixture, and mock strategy.

Owns:

- `RADP-*`
- future `NADAPT-*`, `NINTEGRATION-*`, `NDATA-PROVIDER-*`, `NEXCHANGE-*`,
  `NSANDBOX-*`, `NDB-*`, `NDEFI-*`

May modify:

- `crates/adapters/**`
- `crates/network/**`
- `crates/infrastructure/**`
- `crates/persistence/**`
- `crates/serialization/**`
- `docs/integrations/**`
- `tests/integration/**`
- `examples/adapters/**`

May not modify unless explicitly authorized:

- `backlog/**`
- `state/**`
- `.agentflow/policies/**`
- `crates/core/**` large refactors
- `crates/pyo3/**`
- `python/**`
- `nautilus_trader/**`

Required evidence:

- Adapter inventory or scoped adapter decision.
- Fixture, mock, schema, dry-run, or sandbox validation.
- No hardcoded secret report.
- Clear supported, deferred, or removed classification.

### Verification & Release Gatekeeper

Purpose: QA, golden trace, regression evidence, CI policy, release checklist,
removal gates, and high-risk veto.

Owns:

- `RTRACE-*`
- `RREL-*`
- CI and QA tasks
- future `NQA-*`, `NTRACE-*`, `NCI-*`, `NREL-*`, `NGATE-*`

May modify:

- `tests/**`
- `tests/golden/**`
- `scripts/ai/**`
- `scripts/verify/**`
- `.github/workflows/**`
- `docs/testing/**`
- `docs/release/**`
- `release/**`
- `CHANGELOG*`
- `MIGRATION*`

May not modify unless explicitly authorized:

- core implementation files
- adapter implementation files
- task status history
- scope decision records

Authority:

- May block merge.
- May require more evidence.
- May mark `QA_FAILED`.
- May block removal and release gates.
- May require explicit scope decision for high-risk work.

Rules:

- Must not approve its own implementation PR.
- Must not ignore failed tests.
- Must not mark missing evidence as passed.
- Must not treat `QA_PASSED` as `DONE`.
- Must not delete tests to satisfy release.

## State Machine

Allowed task states:

```text
TODO
READY
LEASED
RUNNING
PR_OPEN
QA_REQUIRED
QA_PASSED
QA_FAILED
NEEDS_CHANGES
BLOCKED
REVIEW_REQUIRED
MERGED
DONE
SCOPE_DECISION_REQUIRED
DEFERRED_BY_SCOPE_DECISION
FAILED
CANCELLED
SUPERSEDED
```

Rules:

- `QA_PASSED` is not `DONE`.
- `BLOCKED` is not `DONE`.
- `DONE` requires QA evidence, review evidence, and merged PR evidence unless a
  task explicitly documents that it is local-only.
- High-risk tasks must stop at `REVIEW_REQUIRED` before merge.
- Critical removal and release tasks require explicit gatekeeper approval.

## Risk Levels

Low risk: docs, examples, task metadata, non-runtime scripts, adapter inventory.

Medium risk: Rust CLI, examples that touch runtime, adapter mock tests, CI,
Cargo feature cleanup.

High risk: workspace restructuring, runtime logic, adapter behavior,
persistence format, feature flag behavior.

Critical risk: removing Python, PyO3, Cython, `build.py`, `pyproject.toml`,
release contract changes, task graph gate changes, release tags, and production
adapter behavior changes.

## Python / PyO3 / Cython Gate

No role may remove `python/**`, `nautilus_trader/**`, `crates/pyo3/**`,
`build.py`, or `pyproject.toml` until the scope decision explicitly approves the
Rust-only route and the release gatekeeper records approval. Until then, NTPRO
is treated as Rust-first cutover work, not final Rust-only removal.
