# RREL-005 Release Candidate Tag Plan

Date: 2026-06-02
Executor: Codex
Task ID: RREL-005 / RREL-008 refresh

## Scope

This document prepares the Rust-only release candidate tag plan. It does not
create a tag, publish a release, or mark the cutover complete.

## Current Tag Decision

Do not create a Rust-only release candidate tag from the current state.

The tag plan is blocked until `scripts/ai/verify_release.sh` passes all phases,
including strict final golden trace replay, and the human owner approves the
RREL-007 signoff packet.

## Proposed Tag Shape

When the blockers are cleared, use a tag format that makes the release
candidate explicit:

```text
ntpro-rust-only-rc.<N>
```

The final `<N>` value must be selected by the human owner at tag time.

## Required Preconditions

Before creating any release candidate tag:

1. `scripts/ai/verify_release.sh` passes.
2. `scripts/ai/check_rust_only_runtime.sh` passes.
3. `scripts/ai/check_cython_removed.sh` passes.
4. The strict final golden trace replay gate passes or is explicitly scoped by
   the release gatekeeper.
5. RREL-006/RREL-008 evidence records green final release verification.
6. RREL-007 owner signoff packet is reviewed and explicitly signed by the
   human owner.
7. RREL-008 is explicitly approved to mark the cutover complete.
8. `main` is clean, current, and protected by the agreed release gate rules.

## Tag Procedure After Approval

Only after all preconditions are satisfied:

1. Fetch and verify `main` points at the approved release commit.
2. Re-run the final release verification commands locally.
3. Create an annotated tag with a message that references the release evidence
   packet and owner signoff.
4. Push the tag to `origin`.
5. Create the GitHub release from that exact tag.
6. Attach or link the release evidence documents.

## Rollback And Abort Rules

- If any final gate fails, abort tag creation.
- If owner signoff is missing or ambiguous, abort tag creation.
- If the working tree is dirty, abort tag creation.
- If RREL-008 is still paused, abort tag creation.
- If a tag is created incorrectly, do not force-update it without explicit
  owner approval; create a superseding release candidate tag instead.

## Current Recommendation

Keep this as a draft plan only. The Rust-only runtime and Cython removal checks
now pass, but the final release command is still blocked at strict golden trace
replay wiring.
