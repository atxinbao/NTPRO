# v0.17.1 Readiness Report - Release Guard Alignment

Date: 2026-06-26
Executor: Codex
Status: READY_FOR_REVIEW

## Summary

v0.17.1 is scoped as release evidence hardening for the already published
`ntpro-rust-only-v0.17.0` milestone. This readiness note aligns the patch
candidate with both required release guards.

Plain Chinese summary: v0.17.1 这里只做发版证据加固，不扩展交易能力。检查项必须同时覆盖
`release-surface-current-guard` 和 `release-publication-guard`。

## Scope

```text
current formal release = ntpro-rust-only-v0.17.0
target patch release = ntpro-rust-only-v0.17.1
capability_expansion = none_patch_hardening_only
production order submission = not included
production order mutation = not included
actual cancel send = not included
automatic remediation = disabled
Dashboard order controls = disabled
Dashboard cancel controls = disabled
```

## Release Guard Accounting

```text
release-surface-current-guard = required
release-publication-guard = required
```

`release-surface-current-guard` proves the release surface points at the current
formal v0.17.0 release. `release-publication-guard` proves the GitHub Release
publication state is present and current for `ntpro-rust-only-v0.17.0`.

Hosted release evidence from
`https://github.com/atxinbao/NTPRO/actions/runs/28180391200` recorded 49/49
PASS, including both `release-surface-current-guard = PASS` and
`release-publication-guard = PASS`.

## Release Manifest

`scripts/ai/verify_v171_release_hardening.sh` writes the machine-readable
manifest to:

```text
target/ntpro-v171/v0_17_1_release_manifest.json
```

The manifest records product version, release tag, current published release
tag, commit, tree, Cargo workspace version, capability, patch-only capability
expansion, tracked dirty state, gate status, generated timestamp, release
binary paths, binary sha256 values, binary byte counts, CLI version output,
build timestamp, source commit, and source tree.

Release mode rejects missing release binaries and rejects non-`target/release`
binaries unless the diagnostic override is set explicitly.

## verify_fast Boundary

`verify_fast alone does not prove compile/static-check coverage`; it is a fast
local smoke check for toolchain and formatting by default. Release evidence must
use `scripts/ai/verify_release.sh v171-release-hardening`, which performs the
explicit release binary build/provenance checks unless diagnostic reuse is
requested by environment override.

## Validation Plan

```text
scripts/ai/verify_release.sh v171-release-hardening
NTPRO_RELEASE_GATE=1 scripts/ai/verify_v171_release_hardening.sh
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_fast.sh
git diff --check
```

## Boundary

This task does not publish a tag or GitHub Release. It only records the guard
requirements for the future v0.17.1 patch hardening path.
