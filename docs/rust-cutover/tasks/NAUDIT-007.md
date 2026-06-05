# NAUDIT-007 - Unsafe and plugin audit register

Milestone: v0.2.0 Audit Backlog
Priority: P2
Default role: Verification & Release Gatekeeper
Risk: medium

## Goal

Create an unsafe and plugin audit register before plugin functionality is
treated as a productized extension surface.

## Scope

- Inventory unsafe blocks relevant to actor registry, plugin loading, FFI, and
  ABI boundaries.
- Inventory plugin loader productization risks:
  - path trust;
  - optional SHA-256 verification;
  - ABI manifest compatibility;
  - panic boundary;
  - cancellation and unload behavior.
- Define preconditions before plugins can be documented as stable product
  functionality.

## Likely files

- `crates/common/src/actor/registry.rs`
- `crates/plugin/`
- `crates/live/src/node.rs`
- `docs/rust-cutover/verification/`
- `docs/rust-cutover/evidence/NAUDIT-007.md`

## Non-goals

- Do not rewrite plugin loading in this audit-register task.
- Do not add a new plugin system.
- Do not mark plugins product-ready without follow-up implementation evidence.

## Dependencies

- none

## Acceptance criteria

- Unsafe/plugin risk register exists.
- Each high-impact unsafe/plugin area has owner, status, required evidence, and
  productization gate.
- Follow-up implementation tasks are split out if needed.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NAUDIT-007.md`.
