# Rust Cutover Versioning Surface

Date: 2026-06-27
Executor: Codex

The authoritative NTPRO versioning reference is `../versioning.md`. This file
exists for rust-cutover release-surface validation commands and records the
current cutover-facing release summary without duplicating the full versioning
matrix.

## Current Release Surface

```text
current release = v0.18.0
current source tag = ntpro-rust-only-v0.18.0
current capability = Owner-Approved Cancel Recovery Preview
next patch = v0.18.1 Release Surface & Provenance Hardening Patch
next capability = v0.19.0 Owner-Approved Single-Shot Actual Cancel
```

## Boundary

v0.18.0 remains preview-only. It records cancel recovery intent, owner approval
lifecycle, preview request/response evidence, post-cancel readback contracts,
failure and rollback evidence, release gates, and Dashboard diagnostics. It
does not include actual cancel send, automatic cancel, retry/replace/amend/
correction/flatten/remediation, strategy-driven production execution,
multi-account or multi-venue execution, real-funds proof in CI, or Dashboard
order/cancel controls.

v0.18.1 is the release surface and provenance hardening patch for this
preview-only baseline. v0.19.0 is the follow-up capability line for
owner-approved single-shot actual cancel.
