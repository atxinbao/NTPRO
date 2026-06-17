# V090 Auto-Merge Risk Sync Evidence

Date: 2026-06-18
Executor: Codex

## Goal

Adjust the V090 Strategy Runtime Foundation task package so implementation tasks
can run through the normal auto-merge workflow while preserving the explicit
release-approval boundary.

## Changes

- `V090-002`, `V090-003`, `V090-004`, `V090-006`, `V090-007`, `V090-008`, and
  `V090-009` are now scoped as medium-risk offline/shadow automation slices.
- `EPIC-V090` now documents that `V090-000` through `V090-013` may use
  auto-merge after local validation and hosted smoke checks pass.
- `V090-014` remains a release-closure task and must not create a tag or publish
  a GitHub Release without explicit owner approval.

## Shrimp Queue

- Queue path: `/Users/mac/.codex/shrimp-data/NTPRO/tasks.json`
- Backup path:
  `/Users/mac/.codex/shrimp-data/NTPRO/tasks_before_V090_auto_merge_risk_sync_20260617T174928Z.json`
- Result:
  - `V090-000` through `V090-013`: medium risk
  - `V090-014`: critical release closure, manual-gated

## Validation

- `jq empty /Users/mac/.codex/shrimp-data/NTPRO/tasks.json`: PASS
- V090 queue risk scan: PASS

## Behavior Impact

No runtime behavior changes. This is task metadata and documentation only.

## Public API Impact

No public API impact.

## Migration Note

No migration note required.

## Rollback Plan

Revert this PR for repository docs. Restore the Shrimp queue from the recorded
backup path if the local queue must be rolled back.
