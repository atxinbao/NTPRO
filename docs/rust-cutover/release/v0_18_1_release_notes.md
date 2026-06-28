# v0.18.1 Release Notes - Published Provenance Hardening

Date: 2026-06-27
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.18.1`
Release name: `NTPRO Rust-only v0.18.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.18.1`
Release commit: `c395e71960255fefbf4100654fd53ce2bf33a08f`
Published at: `2026-06-27T21:06:12Z`

## Summary

v0.18.1 is the Release Surface & Provenance Hardening patch for the published
`ntpro-rust-only-v0.18.0` Owner-Approved Cancel Recovery Preview release. It
does not expand the v0.18.0 capability boundary. It is now published as the
formal `ntpro-rust-only-v0.18.1` prerequisite release for the v0.19 actual
cancel line.

Plain Chinese summary: v0.18.1 是发版面和 provenance 加固补丁。大白话：补 binary、
commit、tag、toolchain、manifest 的对应关系，让后续 v0.19 开始前先证明当前发布证据可信；
已经发布 v0.18.1 tag / GitHub Release，但不新增真实撤单。

## Required Release Gates

Release evidence must include:

```text
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_release.sh v18-strict-provenance
scripts/ai/verify_release_strict.sh v18
```

The default `scripts/ai/verify_fast.sh` command is intentionally excluded from
this release-evidence list. It remains a local fast smoke for pinned toolchain
and `cargo fmt --check` only; it is not a release gate, not compile/test
coverage, and not strict provenance evidence.

The strict provenance gate writes or verifies:

```text
docs/rust-cutover/release/v0_18_1_release_manifest.json
target/ntpro-v181/v0_18_strict_release_manifest.json
```

The docs release manifest records the v0.18.1 release contract: product
version, v0.18.0 baseline release/tag/commit, planned and actual patch tag
fields, required release gates, capability boundary, and no-actual-cancel
flags. The generated strict provenance manifest binds that contract to the
target binary path, binary sha256, binary byte count, source commit, source
tree, baseline release tag, baseline release commit, `cargo --version`, and
`rustc --version`.

## Dashboard Diagnostics Regression Coverage

V181-007 adds v0.18 Dashboard cancel recovery regression tests for missing
artifacts, schema mismatches, source commit/tag provenance mismatches, stale
artifacts, and forbidden cancel/Dashboard approval flags. The panel must report
degraded or boundary violation states for those cases and must not report
`healthy` or `production_cancel_recovery_ready`.

## What Did Not Change

```text
capability_expansion = none_patch_hardening_only
actual cancel send = not included
automatic cancel = disabled
automatic remediation = disabled
Dashboard order controls = disabled
Dashboard cancel controls = disabled
Dashboard auto-approval controls = disabled
v0.18.1 tag publication = completed
GitHub Release publication = completed
```

## Publication Status

```text
release status = RELEASED
release tag = ntpro-rust-only-v0.18.1
release name = NTPRO Rust-only v0.18.1
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.18.1
release commit = c395e71960255fefbf4100654fd53ce2bf33a08f
published at = 2026-06-27T21:06:12Z
GitHub Release draft = false
GitHub Release prerelease = false
```

The publication changes release evidence only. It does not add actual cancel
send, automatic cancel, automatic remediation, Dashboard cancel controls,
production order mutation, or binary asset upload.

## Rollback

Rollback is a documentation and evidence rollback only for this draft: revert
the strict provenance gate PR to remove the strict gate script, manifest
requirement, and V181-004 evidence.
