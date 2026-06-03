# RREL-005 Release Candidate Tag Plan

Date: 2026-06-03
Executor: Codex
Task ID: RREL-005 / RREL-008 / RREL-009

## Scope

This document records the Rust-only release candidate tag plan and the first
completed tag-only release-candidate action. It does not publish a GitHub
Release.

## Current Tag Decision

The first Rust-only release candidate tag has been created:

```text
ntpro-rust-only-rc.1
```

RREL-009 made `scripts/ai/verify_release.sh` pass, and the human owner approved
Rust-only cutover completion on 2026-06-03. After RREL-008 was merged, the
human owner separately approved `ntpro-rust-only-rc.1` as a tag-only action.

The tag points at commit `a886e2ac3682247b5e542599fb8dd219a6b9cf1c`.

No GitHub Release has been published.

## Proposed Tag Shape

Future release-candidate tags should keep the same explicit format:

```text
ntpro-rust-only-rc.<N>
```

The final `<N>` value must be selected by the human owner at tag time.

## Required Preconditions

Before creating any future release candidate tag:

1. `scripts/ai/verify_release.sh` passes.
2. `scripts/ai/check_rust_only_runtime.sh` passes.
3. `scripts/ai/check_cython_removed.sh` passes.
4. The strict final golden trace replay gate passes or is explicitly scoped by
   the release gatekeeper.
5. RREL-008 evidence records green final release verification and owner
   signoff.
6. RREL-008 completion PR is reviewed and merged.
7. The human owner explicitly approves tag creation and selects the final
   `<N>` value.
8. `main` is clean, current, and protected by the agreed release gate rules at
   tag time.

## Future Tag Procedure After Approval

Only after all preconditions are satisfied:

1. Fetch and verify `main` points at the approved release commit.
2. Re-run the final release verification commands locally.
3. Create an annotated tag with a message that references the release evidence
   packet and owner signoff.
4. Push the tag to `origin`.
5. Publish a GitHub Release only if the human owner explicitly approves release
   publication.
6. Attach or link the release evidence documents when a GitHub Release is
   published.

## Rollback And Abort Rules

- If any final gate fails, abort tag creation.
- If owner signoff is missing or ambiguous, abort tag creation.
- If the working tree is dirty, abort tag creation.
- If RREL-008 is not merged and marked complete, abort tag creation.
- If a tag is created incorrectly, do not force-update it without explicit
  owner approval; create a superseding release candidate tag instead.

## Current Recommendation

`ntpro-rust-only-rc.1` is complete as a tag-only release candidate.

Keep GitHub Release publication paused until the public README, release notes,
GitHub checks, Rust CLI entrypoint evidence, and repository language display are
reviewed after RC cleanup.
