# RHARD-001 Post-Release Gap List Evidence

Date: 2026-06-04
Executor: Codex
Task ID: RHARD-001
Risk: low

## Scope

RHARD-001 inventories remaining gaps after the formal
`ntpro-rust-only-v0.1.0` release. It is documentation-only and does not
implement any listed gap.

## Context Reviewed

- `docs/rust-cutover/scope/v0_2_0_roadmap.md`
- `docs/rust-cutover/tasks/RHARD-001.md`
- `crates/cli/`
- `examples/rust/`
- `crates/backtest/examples/`
- `crates/live/examples/`
- `docs/getting_started/`
- `docs/how_to/`
- `docs/tutorials/`
- `docs/integrations/`
- `docs/rust-cutover/inventory/`
- `docs/rust-cutover/evidence/`
- `scripts/ai/`

## Changes

- Added `docs/rust-cutover/post-release-gap-list.md`.
- Grouped remaining work by CLI, examples, docs, adapters, verification, and
  architecture.
- Mapped each gap to the next v0.2.0 task, owner role, risk, and execution
  status.
- Separated known blockers and deferred work from executable v0.2.0 tasks.

## Commands Run

```bash
find crates/cli -maxdepth 3 -type f | sort | head -80
find examples/rust crates/backtest/examples crates/live/examples -maxdepth 3 -type f 2>/dev/null | sort
find docs -maxdepth 3 -type f | sort | rg "(getting_started|how_to|tutorials|integrations|architecture|rust-cutover)" | head -200
find scripts/ai -maxdepth 2 -type f | sort
find docs/rust-cutover/inventory docs/rust-cutover/evidence -maxdepth 2 -type f | sort | tail -120
```

Result: inventory inputs were available and sufficient for a docs-only gap
list.

Validation commands:

```bash
git diff --check
scripts/ai/verify_fast.sh
scripts/ai/validate_agentflow_roles.py
```

Result: all passed.

`verify_fast` output summary:

```text
== verify_fast: toolchain ==
== verify_fast: rust fmt ==
== verify_fast: cargo check skipped; set VERIFY_FAST_CARGO_CHECK=1 to run the legacy mixed-workspace check ==
== verify_fast: clippy skipped; set VERIFY_FAST_CLIPPY=1 to run it in fast mode ==
== verify_fast complete ==
```

## Behavior Impact

No trading behavior changed. This task only records post-release planning
gaps.

## Rollback Plan

Revert the PR to remove the gap list and RHARD-001 evidence.
