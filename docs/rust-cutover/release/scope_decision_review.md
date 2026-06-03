# RREL-003 Scope Decision Review

Date: 2026-06-03
Executor: Codex
Task ID: RREL-003 / RREL-008 / RREL-009

Updated: 2026-06-04
Executor: Codex
Follow-up ID: RC-CLEANUP-001

## Scope

This review checks whether owner-visible scope decisions exist for the
Rust-only cutover completion record.

No new scope decision is approved by this document. It records the current
decision coverage after RREL-009 passed final local release verification and
the human owner approved completion on 2026-06-03.

## Reviewed Sources

- `docs/rust-cutover/scope/SCOPE_DECISIONS.md`
- `docs/rust-cutover/CONTRACT.md`
- `.agentflow/policies/gates.yaml`
- `docs/rust-cutover/migration/final_rust_only_removal_gate.md`
- `docs/rust-cutover/migration/rust_only_migration_guide.md`
- `docs/rust-cutover/release/rust_only_release_notes.md`
- `docs/rust-cutover/release/final_release_verification.md`
- `docs/rust-cutover/release/human_owner_signoff_packet.md`
- `docs/rust-cutover/evidence/RREM-022.md`
- `docs/rust-cutover/evidence/RREL-008.md`
- `docs/rust-cutover/evidence/RREL-009.md`

## Decision Coverage

| Area | Owner-visible decision | Review result |
| --- | --- | --- |
| Product route | `CONTRACT.md` defines Rust-only as the target final product state. | Covered. |
| Python/PyO3/Cython removal | `SD-001` gates removal until product, runtime, adapter, QA, and release evidence are complete. | Covered; removal evidence is recorded through RREM-022. |
| Final removal gate | `RREM-*` and RREL evidence record the staged removal and release gate result. | Covered. |
| Migration guidance | RREL-001 documents the target migration path and current Rust-only state. | Covered. |
| Release notes | RREL-002 documents breaking changes and replacement workflows. | Covered. |
| Final release verification | RREL-009 records green final local release verification. | Covered. |
| Owner signoff | RREL-007/RREL-008 records approval by atxinbao on 2026-06-03. | Covered. |
| Completion declaration | RREL-008 records completion approval and stops at PR review. | Covered, review-gated. |
| Tag/release publication | RREL-005 kept tag and GitHub Release publication as separate manual actions. `ntpro-rust-only-rc.1`, `ntpro-rust-only-rc.2`, `ntpro-rust-only-rc.3`, and `ntpro-rust-only-v0.1.0` were later approved as explicit owner-controlled publication steps. | Tags covered; `ntpro-rust-only-v0.1.0` is the formal Rust-only GitHub Release target after explicit owner approval. |

## P0/P1 Deferral Review

The previous P0/P1 release blockers are resolved for the RREL-008 completion
record:

- Rust-only removal evidence is recorded.
- `scripts/ai/verify_release.sh` passed after RREL-009.
- `scripts/ai/check_rust_only_runtime.sh` passed.
- `scripts/ai/check_cython_removed.sh` passed.
- Final golden trace release mode passed through
  `docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json`.
- Human owner signoff is recorded in the signoff packet.

Remaining controls are not completion blockers:

- RREL-008 has been reviewed and merged.
- `ntpro-rust-only-rc.3` has been created and published as the final pre-release
  candidate after separate owner approval.
- `ntpro-rust-only-v0.1.0` has been selected as the first formal Rust-only
  GitHub Release target after separate owner approval.

## Review Decision

Scope decision coverage is sufficient for the completed Rust-only cutover and
the `ntpro-rust-only-v0.1.0` formal release target.

This review still requires explicit owner approval for any later release
publication.
