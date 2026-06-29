# Rust Cutover Versioning Surface

Date: 2026-06-27
Executor: Codex

The authoritative NTPRO versioning reference is `../versioning.md`. This file
exists for rust-cutover release-surface validation commands and records the
current cutover-facing release summary without duplicating the full versioning
matrix.

## Current Release Surface

```text
current release = v0.19.0
current source tag = ntpro-rust-only-v0.19.0
current capability = Owner-Approved Single-Shot Actual Cancel
next patch = v0.19.1 Actual Cancel Release Closeout & Provenance Hardening Patch
next capability = v0.20.0 Owner-Approved Production Order Lifecycle Foundation
```

## Boundary

v0.19.0 remains actual-cancel-only. It records one manual owner approval, one
order, one venue, one execution attempt, risk gate evidence, adapter boundary
evidence, post-cancel readback, failure evidence, release gates, golden traces,
and Dashboard read-only audit diagnostics. It does not include production order
submit lifecycle, automatic cancel, bulk cancel, retry/replace/amend/
correction/flatten/remediation, strategy-driven production execution,
multi-account or multi-venue execution, real-funds proof in CI, or Dashboard
order/cancel controls.

v0.19.1 is the release closeout and provenance hardening patch for this
actual-cancel-only baseline. v0.20.0 is the follow-up capability line for
Owner-Approved Production Order Lifecycle Foundation.
