# v0.17.1 Release Notes - Release Evidence Hardening

Date: 2026-06-26
Executor: Codex
Status: DRAFT_NOT_PUBLISHED

## Summary

v0.17.1 is a release-evidence hardening patch for the v0.17.0 production
reconciliation and orphan recovery milestone.

Plain Chinese summary: 这是 v0.17.1 的草稿 release notes。它只加固发版证据和 provenance，
不新增生产下单、撤单、自动补救或 Dashboard 控制能力。

## What Changed

- V170-009 evidence now reflects the completed `ntpro-rust-only-v0.17.0`
  GitHub Release and hosted release gate result.
- Added `v171-release-hardening` to local release verification and release-tag
  workflow routing.
- Added a machine-readable release provenance manifest for the future
  `ntpro-rust-only-v0.17.1` patch target.
- Release binary provenance now records path, bytes, sha256, version output,
  build timestamp, and source commit.
- v0.17 source artifact references now retain legacy `fnv1a64` and add
  `sha256`, `bytes`, `source_command`, `source_commit`, and
  `source_release_tag`.
- The v0.17 Dashboard reconciliation/orphan panel now reports
  `missing_artifacts`, `schema_mismatches`, `provenance_issues`, and
  `stale_evidence_issues`.

## What Did Not Change

```text
capability_expansion = none_patch_hardening_only
production order submission = not included
production order mutation = not included
actual cancel send = not included
automatic cancel = disabled
automatic remediation = disabled
Dashboard order controls = disabled
Dashboard cancel controls = disabled
```

## Validation

Release validation must include:

```text
scripts/ai/verify_release.sh v171-release-hardening
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh release-publication-guard
```

`verify_fast alone does not prove compile/static-check coverage`; it remains a
quick local smoke check and must not be described as full compile/static-check
release proof.

## Publication Status

This document does not publish a GitHub Release. The current formal release is
`ntpro-rust-only-v0.17.0`; the future patch target is
`ntpro-rust-only-v0.17.1`.

