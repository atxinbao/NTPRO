# v0.17.1 Readiness Report - Release Closeout

Date: 2026-06-26
Executor: Codex
Status: READY_FOR_REVIEW

## Summary

v0.17.1 is scoped as patch hardening for the already published
`ntpro-rust-only-v0.17.0` release. It strengthens release evidence,
provenance, and Dashboard diagnostics without expanding the v0.17.0
reconciliation/orphan recovery capability.

Plain Chinese summary: v0.17.1 只做发版可信度补强。大白话：补齐 release guard、
manifest、binary provenance、artifact provenance 和 Dashboard 诊断，不新增交易能力，也不发布
tag 或 GitHub Release。

## Scope Boundary

```text
current formal release = ntpro-rust-only-v0.17.0
target patch release = ntpro-rust-only-v0.17.1
capability_expansion = none_patch_hardening_only
production order submission = not included
production order mutation = not included
network readback execution = not included
actual cancel send = not included
automatic remediation = disabled
Dashboard order controls = disabled
Dashboard cancel controls = disabled
tag publication = not included
GitHub Release publication = not included
```

## V171 Task Accounting

| Task | Issue / PR | Status | Evidence | Release impact |
| --- | --- | --- | --- | --- |
| V171-001 | #531 / PR #551 | MERGED | `docs/rust-cutover/evidence/V171-001.md` | Syncs formal v0.17.0 evidence after release publication. |
| V171-002 | #532 / PR #552 | MERGED | `docs/rust-cutover/evidence/V171-002.md` | Aligns release-publication guard evidence. |
| V171-003 | #533 / PR #553 | MERGED | `docs/rust-cutover/evidence/V171-003.md` | Adds v0.17.1 release provenance manifest gate. |
| V171-004 | #534 / PR #554 | MERGED | `docs/rust-cutover/evidence/V171-004.md` | Proves release binaries come from the current commit. |
| V171-005 | #535 / PR #555 | MERGED | `docs/rust-cutover/evidence/V171-005.md` | Clarifies that `verify_fast` is not release proof. |
| V171-006 | #536 / PR #556 | MERGED | `docs/rust-cutover/evidence/V171-006.md` | Extends artifact refs with `sha256`, `bytes`, `source_command`, `source_commit`, and `source_release_tag`. |
| V171-007 | #537 / PR #557 | MERGED | `docs/rust-cutover/evidence/V171-007.md` | Adds explicit read-only Dashboard diagnostics for missing, schema-mismatched, provenance-mismatched, and stale artifacts. |
| V171-008 | #538 / current PR | READY_FOR_REVIEW | `docs/rust-cutover/evidence/V171-008.md` | Closes v0.17.1 readiness notes, release notes, task accounting, and rollback evidence. |

## Release Guard Accounting

```text
release-surface-current-guard = required
release-publication-guard = required
v171-release-hardening = required
```

Hosted release evidence from
`https://github.com/atxinbao/NTPRO/actions/runs/28180391200` recorded 49/49
PASS for the current formal `ntpro-rust-only-v0.17.0` release, including
`release-surface-current-guard = PASS` and `release-publication-guard = PASS`.

## Release Manifest

`scripts/ai/verify_v171_release_hardening.sh` writes the machine-readable
manifest to:

```text
target/ntpro-v171/v0_17_1_release_manifest.json
```

The manifest records:

```text
product_version = v0.17.1
release_tag = ntpro-rust-only-v0.17.1
current_published_release_tag = ntpro-rust-only-v0.17.0
capability_expansion = none_patch_hardening_only
release_binaries = nautilus, ntpro-node
binary_provenance = path, bytes, sha256, version_output, build_timestamp, source_commit, source_tree, source_dirty
artifact_provenance = V170-009 plus V171-001 through V171-008 evidence/docs/scripts
```

Artifact provenance includes `sha256`, `bytes`, `source_command`,
`source_commit`, and `source_release_tag`. v0.17 source artifact refs also
retain the legacy `fnv1a64` hash for compatibility.

## Dashboard Diagnostics

V171-007 keeps the Dashboard read-only while making degraded evidence explicit:

```text
missing_artifacts_displayed = true
schema_expected_actual_displayed = true
provenance_sha256_or_bytes_mismatch_displayed = true
stale_artifacts_displayed = true
dashboard_order_controls_enabled = false
dashboard_cancel_controls_enabled = false
```

## Validation Plan

```text
bash -n scripts/ai/verify_v171_release_hardening.sh
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_release.sh v171-release-hardening
scripts/ai/verify_fast.sh
git diff --check
```

## Rollback Plan

Revert the V171 task PRs in reverse order if the patch hardening path must be
abandoned. Reverting V171-008 removes only closeout documentation and manifest
artifact accounting. It does not change trading behavior, publish a release, or
touch production exchange state.
