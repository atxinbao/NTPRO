# RHARD-001 - Post-release gap list

Milestone: v0.2.0 Hardening
Priority: P0
Default role: Rust Product Surface
Risk: low

## Goal

List remaining gaps after v0.1.0.

## Scope

- Inventory CLI gaps.
- Inventory examples gaps.
- Inventory docs gaps.
- Inventory adapter gaps.
- Inventory verification gaps.
- Inventory architecture gaps.

## Likely files

- `docs/rust-cutover/post-release-gap-list.md`
- `docs/rust-cutover/evidence/RHARD-001.md`

## Non-goals

- Do not implement the listed gaps.
- Do not change runtime or adapter behavior.
- Do not create release tags or GitHub Releases.

## Dependencies

- `RHARD-000`

## Acceptance criteria

- The post-release gap list exists.
- Each gap is assigned an area and a proposed follow-up owner.
- Blockers and deferrals are clearly separated from executable v0.2.0 work.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RHARD-001.md`.
