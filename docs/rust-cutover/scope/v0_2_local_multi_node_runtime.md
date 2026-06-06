# NTPRO v0.2 Scope Decision - Local Multi-Node Runtime Foundation

Date: 2026-06-06
Executor: Codex
Task ID: V02-001
Status: active scope decision

## Decision

NTPRO v0.2 is scoped as:

```text
Local Multi-Node Runtime Foundation
```

This scope starts only after `DRG-010` recorded:

```text
Design Readiness Gate: PASS
```

The v0.2 goal is to make local Rust node processes manageable through a small
supervisor foundation. It is not a Dashboard release, not a production exchange
release, and not a v0.2 tag decision.

## Why This Supersedes The Older Roadmap

The older `docs/rust-cutover/scope/v0_2_0_roadmap.md` described a broad
Rust-only product hardening roadmap. Much of that work has already been handled
through post-release hardening, audit tasks, and the DRG gate sequence.

Continuing to use the older roadmap as the executable v0.2 task source would
mix completed public-surface hardening, future Dashboard contracts, and runtime
foundation work into one track. That would make automation ambiguous and could
lead agents to start Dashboard or control API work before local node runtime
evidence exists.

Therefore, the old roadmap is retained as historical planning context only. The
active v0.2 execution source is this scope decision plus `V02-001` through
`V02-010`.

## In Scope

- Define a local node and supervisor product contract.
- Define stable node status DTOs.
- Provide a sandbox-only real `LiveNode` start/stop path for local node use.
- Implement a local supervisor registry.
- Implement local supervisor start, stop, and status flows.
- Expose per-node log and minimal metrics JSON artifacts.
- Add CLI commands that operate through the local supervisor path.
- Prove a two-node local sandbox smoke.
- Produce a final v0.2 readiness report with strict PASS/FAIL.

## Out Of Scope

- Dashboard UI implementation.
- Production real-exchange live trading operation.
- Manual order entry.
- Strategy parameter hot reload.
- VWAP/POV execution algorithm productization.
- Nexus-like or distributed multi-server communication.
- Multi-user permissions.
- Docker delivery as a v0.2 requirement.
- Release tags or GitHub Releases.
- New Python, PyO3, or Cython product surfaces.

## Product Boundary

v0.2 may introduce user-visible CLI commands, but only for local runtime
foundation behavior. User-facing text must not claim:

- Dashboard availability;
- production exchange readiness;
- remote/distributed node orchestration;
- manual trading operations;
- release/tag readiness.

## Task Sequence

```text
V02-001 Scope decision and roadmap rewrite
  -> V02-002 Node and Supervisor product contract
  -> V02-003 Node status DTOs
  -> V02-004 ntpro-node sandbox-only real LiveNode start/stop
  -> V02-005 ntpro-supervisor registry
  -> V02-006 supervisor start stop status
  -> V02-007 logs and minimal metrics JSON
  -> V02-008 CLI controls supervisor
  -> V02-009 two-node local smoke test
  -> V02-010 v0.2 readiness report
```

## Evidence Rules

Every V02 task must record evidence under `docs/rust-cutover/evidence/`.

Runtime-facing V02 tasks must use local sandbox, mock, or fixture evidence.
High-risk tasks must stop for review before merge according to
`docs/rust-cutover/TASK_EXECUTION.md`.

## Release Boundary

V02 completion is not release approval. `V02-010` may declare v0.2 readiness only
after it cites evidence for `V02-001` through `V02-009`; tag creation or GitHub
Release publication still requires a separate explicit user approval.
