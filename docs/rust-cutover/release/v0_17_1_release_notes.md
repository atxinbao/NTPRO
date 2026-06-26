# v0.17.1 Release Notes - Draft Closeout

Date: 2026-06-26
Executor: Codex
Status: DRAFT_NOT_PUBLISHED

## Summary

v0.17.1 is a patch-hardening draft for release evidence. It does not expand the
v0.17.0 production reconciliation and orphan recovery capability, and it does
not publish a tag or GitHub Release.

Plain Chinese summary: v0.17.1 是发版证据加固草稿。大白话：只补可信度、追溯性和
Dashboard 诊断，不新增下单、撤单、联网回查或自动补救能力。

## What Changed

- Release-facing docs now account for V171-001 through V171-008.
- Release evidence requires `release-surface-current-guard`,
  `release-publication-guard`, and `v171-release-hardening`.
- Added `target/ntpro-v171/v0_17_1_release_manifest.json` as the generated
  machine-readable release manifest path.
- Release binary provenance records path, bytes, sha256, CLI version output,
  build timestamp, source commit, source tree, and clean tracked-worktree state.
- v0.17 source artifact references retain legacy `fnv1a64` and add `sha256`,
  `bytes`, `source_command`, `source_commit`, and `source_release_tag`.
- The v0.17.1 release manifest tracks artifact provenance for V170-009 and all
  V171 closeout evidence/docs/scripts through V171-008.
- The Dashboard reconciliation/orphan panel now names missing artifacts, schema
  expected/actual mismatches, provenance hash/byte mismatches, and stale
  evidence while remaining read-only.
- `verify_fast` semantics are documented as fast smoke only, not release proof.

## What Did Not Change

```text
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

## Validation

Release evidence must include:

```text
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_release.sh v171-release-hardening
```

Local closeout validation also includes:

```text
bash -n scripts/ai/verify_v171_release_hardening.sh
scripts/ai/verify_fast.sh
git diff --check
```

## Publication Status

This document does not publish a GitHub Release. The current formal release is
`ntpro-rust-only-v0.17.0`; the patch target remains
`ntpro-rust-only-v0.17.1` until explicitly published in a later owner-approved
release step.

## Rollback

Rollback is a documentation and evidence rollback only for this closeout draft:
revert the closeout PR to remove the updated readiness report, release notes,
V171-008 evidence, and manifest artifact accounting.
