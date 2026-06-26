# v0.17.1 Release Notes - Release Guard Alignment

Date: 2026-06-26
Executor: Codex
Status: DRAFT_NOT_PUBLISHED

## Summary

v0.17.1 is a patch-hardening release notes draft for release evidence. It does
not expand the v0.17.0 production reconciliation and orphan recovery capability.

Plain Chinese summary: 这份 v0.17.1 草稿只补发版检查说明。大白话：既要证明 release
surface 当前正确，也要证明 GitHub Release 发布态正确。

## What Changed

- Release-facing docs now require `release-surface-current-guard`.
- Release-facing docs now require `release-publication-guard`.
- Added `target/ntpro-v171/v0_17_1_release_manifest.json` as the generated
  machine-readable release manifest path.
- Release binary provenance now records path, bytes, sha256, CLI version output,
  build timestamp, source commit, source tree, and clean tracked-worktree state.
- v0.17 source artifact references retain legacy `fnv1a64` and add `sha256`,
  `bytes`, `source_command`, `source_commit`, and `source_release_tag`.
- Hosted release evidence references the completed
  `ntpro-rust-only-v0.17.0` release gate run with 49/49 PASS.

## What Did Not Change

```text
capability_expansion = none_patch_hardening_only
production order submission = not included
production order mutation = not included
actual cancel send = not included
automatic remediation = disabled
Dashboard order controls = disabled
Dashboard cancel controls = disabled
```

## Validation

Release evidence must include both:

```text
scripts/ai/verify_release.sh v171-release-hardening
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh release-publication-guard
```

`verify_fast alone does not prove compile/static-check coverage`; it remains a
fast local smoke check by default. The v0.17.1 release proof comes from the
explicit release binary/provenance checks in `v171-release-hardening`.

## Publication Status

This document does not publish a GitHub Release. The current formal release is
`ntpro-rust-only-v0.17.0`; the future patch target remains
`ntpro-rust-only-v0.17.1`.
