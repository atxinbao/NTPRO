# NAUDIT-005 - PostgreSQL cache adapter support classification

Milestone: v0.2.0 Audit Backlog
Priority: P1
Default role: Adapter & Integration
Risk: medium

## Goal

Classify the PostgreSQL cache adapter as supported, experimental, or
unsupported for the v0.2 product surface.

## Scope

- Inventory `not implemented for PostgreSQL cache adapter` operations.
- Review ignored PostgreSQL cache integration tests and schema/FK blockers.
- Decide product status:
  - unsupported;
  - experimental;
  - supported.
- Update docs, release notes, and adapter/support matrix as needed.
- If unsupported or experimental, ensure CLI/docs do not present it as a stable
  product path.

## Likely files

- `crates/infrastructure/src/sql/cache.rs`
- `crates/infrastructure/tests/test_cache_postgres.rs`
- `docs/integrations/`
- `docs/rust-cutover/`

## Non-goals

- Do not implement full PostgreSQL cache persistence in this classification
  task unless explicitly approved later.
- Do not change database schema as part of documentation-only classification.

## Dependencies

- `NADAPT-001`

## Acceptance criteria

- PostgreSQL cache product status is explicit.
- Any unsupported or experimental boundary is visible in docs.
- Follow-up implementation work is split into separate tasks if needed.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NAUDIT-005.md`.
