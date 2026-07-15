# NTPRO Rust Cutover Pull Request

NTPRO is a Rust-first cutover workspace. Keep every PR bound to one GitHub
issue, one branch, and one review gate.

## Task

- Task ID:
- Source task file:
- Owner role:
- Review role:
- Risk level:
- Branch:
- GitHub issue:
- Backend freeze classification:
- Frozen backend boundary touched:
- Forbidden capability request:
- Backend freeze exception authorization:
- v0.33+ separate scope:

## Summary

<!-- What changed, why it changed, and what behavior is affected. -->

## 大白话说明

<!-- 用中文说明：这次改了什么、没改什么、验证结果、行为影响、是否可以自动合并。 -->

## Files Changed

<!-- List the important changed paths. -->

## Scope Checklist

- [ ] This PR covers one task only.
- [ ] Touched paths match the GitHub issue and task path scope.
- [ ] No unrelated refactors or formatting churn.
- [ ] No forbidden paths were modified.
- [ ] Python, PyO3, or Cython surfaces were not removed unless an explicit scope
      decision and release gate allow it.
- [ ] Trading semantics are unchanged, or golden trace evidence is included.
- [ ] Adapter behavior is unchanged, or fixture/mock evidence is included.
- [ ] Public API behavior is unchanged, or migration notes are included.

## Backend Freeze Declaration

- [ ] This PR is classified as `baseline-preserving-governance`,
      `v33-separately-scoped`, or `backend-freeze-exception`.
- [ ] The v0.32.0 frozen registry, release facts, boundary fields, and protected
      paths touched by this PR are listed above.
- [ ] This PR does not inherit backend go-live, submit, mutation, adapter send,
      live exchange, retry, remediation, recovery, or trading controls from
      v0.32.0.
- [ ] Any v0.33+ scope identifies its owner, module boundary, dependencies, data
      contract, rollback, telemetry, and non-inheritance evidence.
- [ ] Any backend-freeze exception includes explicit authorization, impact,
      rollback, and reconstructable audit evidence.
- [ ] `scripts/ai/verify_release.sh backend-freeze-baseline` passed, or the
      exception evidence explains why the frozen baseline itself is invalid.

## Evidence

- Evidence file:
- Plain Chinese summary:
- Commands run:
- Command result summary:
- Tests added or updated:
- Tests not run and reason:

## Impact

- Runtime behavior impact:
- Public API impact:
- Migration note status:
- Release gate impact:

## Rollback Plan

<!-- Exact revert or rollback steps. -->

## Review Gate

- [ ] Owner role did not approve its own task.
- [ ] Verification/release gatekeeper evidence is present or explicitly not
      required for this risk level.
- [ ] `BLOCKED` is not treated as `DONE`.
- [ ] `QA_PASSED` is not treated as `DONE`.
