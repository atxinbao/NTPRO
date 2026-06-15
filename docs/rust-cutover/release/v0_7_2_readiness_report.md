# NTPRO v0.7.2 Readiness Report

Date: 2026-06-15
Executor: Codex
Milestone: v0.7.2 release wording and evidence patch
Status: RELEASE CANDIDATE ONLY - not tagged, not published

## Decision

Status: READY FOR OWNER RELEASE DECISION AFTER THIS PR MERGES.

`v0.7.2` is a release-surface patch for the already published v0.7.1
read-only Binance testnet proof hardening line. It does not expand the product
capability claim.

This report prepares the release decision package only. It does not create a
tag and does not publish a GitHub Release.

## Plain Chinese Summary

v0.7.2 不是新交易能力版本，只是把 v0.7.1 正式发布后的公开口径和证据补齐。

大白话：v0.7.1 已经发了，后面又补了几处“发布后收口”的问题，包括 README/ROADMAP
当前版本口径、online HTTP 只读探测 artifact notes、以及正式发布闭环证据。v0.7.2
如果发布，也只能说是 wording/evidence patch，不能说成真实下单、实盘、生产交易、
Dashboard 发起联网或 authenticated account access。

## Included Candidate Scope

| Task / PR | Status | Scope |
| --- | --- | --- |
| V072-001 / #326 | merged | Replace v0.7.1 pre-release wording with released wording. |
| V072-002 / #327 | merged | Align README and ROADMAP current release wording to v0.7.1. |
| V072-003 / #328 | merged | Correct explicit opt-in HTTP read-only boundary notes. |
| V072-004 / #329 | merged | Record v0.7.1 hosted gate, tag-triggered gate, and Release URL evidence. |
| V072-005 | this PR | Prepare v0.7.2 candidate notes and readiness closeout. |

## Not Included

`v0.7.2` must not claim:

- testnet order submission;
- order cancel, replace, amend, or live order management;
- authenticated Binance testnet account access;
- account mutation;
- real account reconciliation;
- production Binance connectivity;
- production trading;
- real funds;
- Dashboard network initiation;
- Dashboard credential access;
- prebuilt binary or Docker delivery.

## Release Boundary

```text
v0.7.2 = release wording/evidence patch only
current capability = v0.7 read-only Binance testnet connectivity proof
default mode = offline / CI-safe
manual online HTTP proof = explicit owner opt-in only
manual online proof command = NTPRO_V07_MANUAL_ONLINE=1 NTPRO_ALLOW_TESTNET_NETWORK=1 scripts/ai/verify_v07_manual_online_gate.sh
order submission = not included
real funds = not included
production trading = not included
tag/release = separate owner approval required
```

## Required Verification Before Any v0.7.2 Tag

Before creating any `ntpro-rust-only-v0.7.2` tag, verify the exact release
commit after this PR merges:

```bash
scripts/ai/verify_fast.sh
scripts/ai/verify_v07_default_offline_gate.sh
scripts/ai/verify_v07_manual_online_gate.sh
scripts/ai/verify_release.sh v07-default-offline-gate v07-manual-online-preflight
git diff --check
```

Hosted release gate evidence is required if the owner decides to publish a
formal GitHub Release.

## Current Validation For This Candidate PR

```text
scripts/ai/verify_fast.sh = required
git diff --check = required
text boundary scan = required
```

## Release Decision Checklist

- [ ] Owner approves `ntpro-rust-only-v0.7.2` tag creation.
- [ ] Exact post-merge release commit is recorded.
- [ ] Hosted Release Gate passes on the exact release commit.
- [ ] Tag-triggered Release Gate passes on `ntpro-rust-only-v0.7.2`.
- [ ] GitHub Release body uses `v0_7_2_release_notes.md`.
- [ ] Release body keeps no orders / no real funds / no production trading
      boundary.

## Rollback Plan

If v0.7.2 is not released, keep this report as a prepared candidate package or
revert the V072-005 PR. No runtime behavior depends on it.
