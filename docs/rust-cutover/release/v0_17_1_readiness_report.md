# v0.17.1 Readiness Report - Release Evidence Hardening

Date: 2026-06-26
Executor: Codex
Status: READY_FOR_REVIEW

## Summary

v0.17.1 is a patch-hardening release candidate for release evidence and
provenance. It does not expand the v0.17.0 production reconciliation and orphan
recovery capability.

Plain Chinese summary: v0.17.1 只做发版证据加固。当前正式发布仍是
`ntpro-rust-only-v0.17.0`；目标 patch tag 是 `ntpro-rust-only-v0.17.1`，本任务不发布 tag。

## Scope

```text
current formal release = ntpro-rust-only-v0.17.0
target patch release = ntpro-rust-only-v0.17.1
capability_expansion = none_patch_hardening_only
production order submission = not included
production order mutation = not included
actual cancel send = not included
automatic cancel = disabled
Dashboard order controls = disabled
Dashboard cancel controls = disabled
```

## V171 Task Accounting

| Task | Status | Evidence |
| --- | --- | --- |
| V171-001 Sync V170-009 final release evidence | PASS | `docs/rust-cutover/evidence/V171-001.md` |
| V171-002 Align release-publication guard evidence | PASS | `docs/rust-cutover/evidence/V171-002.md` |
| V171-003 Add release provenance manifest | PASS | `docs/rust-cutover/evidence/V171-003.md` |
| V171-004 Prove release binaries from current commit | PASS | `docs/rust-cutover/evidence/V171-004.md` |
| V171-005 Clarify verify_fast semantics | PASS | `docs/rust-cutover/evidence/V171-005.md` |
| V171-006 Upgrade artifact hash and provenance contract | PASS | `docs/rust-cutover/evidence/V171-006.md` |
| V171-007 Add Dashboard degraded artifact diagnostics | PASS | `docs/rust-cutover/evidence/V171-007.md` |
| V171-008 Prepare v0.17.1 readiness and release closeout | PASS | `docs/rust-cutover/evidence/V171-008.md` |

## Manifest

`scripts/ai/verify_v171_release_hardening.sh` generates:

```text
target/ntpro-v171/v0_17_1_release_manifest.json
schema_version = ntpro.v171_release_provenance_manifest.v1
product_version = v0.17.1
release_tag = ntpro-rust-only-v0.17.1
current_published_release_tag = ntpro-rust-only-v0.17.0
capability_expansion = none_patch_hardening_only
```

The manifest records commit, tree, tracked dirty state, Cargo workspace
version, generated timestamp, gate status, release binary provenance, and
artifact provenance. Release gate mode rejects dirty tracked state.

## Release Guards

```text
release-surface-current-guard = required
release-publication-guard = required
v171-release-hardening = required
```

## verify_fast Boundary

`verify_fast alone does not prove compile/static-check coverage`; it is a fast
local smoke check. Release evidence must use `scripts/ai/verify_release.sh` and
the targeted release stages.

## Validation Plan

```text
bash -n scripts/ai/verify_v171_release_hardening.sh scripts/ai/verify_release.sh
scripts/ai/verify_release.sh v171-release-hardening
cargo test -p nautilus-cli production_mutation_local_order_ledger_links_single_candidate_chain --lib
cargo test -p nautilus-cli production_reconciliation_orphan_artifacts_populate_readonly_dashboard_panel --lib
cargo test -p nautilus-cli production_reconciliation_orphan_missing_artifacts_degrade_panel --lib
cargo test -p nautilus-cli production_reconciliation_orphan_schema_provenance_and_stale_diagnostics_degrade_panel --lib
scripts/ai/verify_fast.sh
git diff --check
```

## Risk And Rollback

Risk is medium because release evidence, release workflow routing, and
Dashboard diagnostics changed. Rollback is to revert the V171 scripts, workflow
stage, source-ref provenance fields, Dashboard diagnostic fields/tests, and
V171 docs. No production trading behavior is changed.

