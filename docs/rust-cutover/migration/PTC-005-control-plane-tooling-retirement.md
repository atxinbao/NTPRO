# PTC-005 Control-Plane Tooling Retirement

Date: 2026-07-16
Executor: Codex

## Decision

GitHub is the only active control plane for NTPRO tasks. The repository no
longer supports local AgentFlow/Shrimp queue mutation, leases, automatic task
branch dispatch, or local PR-to-queue synchronization.

## Replacement Routes

| Retired behavior | Supported route |
| --- | --- |
| claim/release a local lease | one open GitHub issue, `agent-ready`, one `codex/` branch, one PR |
| validate local role/state JSON | issue fields, repository policy, PR template, and review |
| choose and dispatch the next task | `gh issue list`, live dependency check, then `git switch -c` from `origin/main` |
| mirror merged PR into local queue | `gh pr view`, `Closes #...`, and live issue closure |
| regenerate Cython inventory | retired; the cutover-era `cython_inventory.csv` is a frozen historical snapshot |

No custom Rust dispatcher was added because GitHub and standard CLI commands
already provide the required durable state and audit trail.

## Evidence Retention

Historical evidence is retained. Existing `docs/rust-cutover/` task, evidence,
release, migration, governance, and inventory records may continue to mention
the removed tools. `docs/rust-cutover/inventory/cython_inventory.csv` remains
byte-identical and is not a current source scanner.

## Boundaries

This retirement changes repository governance only. It does not modify the
v0.32.0 backend baseline, runtime, public API, trading semantics, adapters, or
forbidden capability flags.
