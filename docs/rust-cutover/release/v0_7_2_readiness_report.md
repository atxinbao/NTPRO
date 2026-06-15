# NTPRO v0.7.2 Readiness Report

Date: 2026-06-15
Executor: Codex
Milestone: v0.7.2 release wording and evidence patch
Status: RELEASED

## Decision

Status: PASS.

`v0.7.2` is a release-surface patch for the already published v0.7.1
read-only Binance testnet proof hardening line. It does not expand the product
capability claim.

The owner-approved release closure created tag `ntpro-rust-only-v0.7.2` and
published the formal GitHub Release after hosted gates passed on the exact
release commit.

## Plain Chinese Summary

v0.7.2 不是新交易能力版本，只是把 v0.7.1 正式发布后的公开口径和证据补齐。

大白话：v0.7.1 已经发了，后面又补了几处“发布后收口”的问题，包括 README/ROADMAP
当前版本口径、online HTTP 只读探测 artifact notes、以及正式发布闭环证据。v0.7.2
现在已经正式发布，但仍只能说是 wording/evidence patch，不能说成真实下单、实盘、
生产交易、Dashboard 发起联网或 authenticated account access。

## Included Scope

| Task / PR | Status | Scope |
| --- | --- | --- |
| V072-001 / #326 | merged | Replace v0.7.1 pre-release wording with released wording. |
| V072-002 / #327 | merged | Align README and ROADMAP current release wording to v0.7.1. |
| V072-003 / #328 | merged | Correct explicit opt-in HTTP read-only boundary notes. |
| V072-004 / #329 | merged | Record v0.7.1 hosted gate, tag-triggered gate, and Release URL evidence. |
| V072-005 / #330 | merged | Prepare v0.7.2 candidate notes and readiness closeout. |
| V080-PRE-001 / #331 | merged | Finalize v0.7.2 release notes as published. |
| V080-PRE-002 | this PR | Finalize v0.7.2 readiness report as released. |

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
tag/release = owner approved and completed
formal tag = ntpro-rust-only-v0.7.2
formal GitHub Release = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.7.2
```

## Release Verification Evidence

The exact release commit was verified locally before tag creation:

```bash
scripts/ai/verify_fast.sh
scripts/ai/verify_v07_default_offline_gate.sh
scripts/ai/verify_v07_manual_online_gate.sh
scripts/ai/verify_release.sh v07-default-offline-gate v07-manual-online-preflight
git diff --check
```

Hosted release gates also passed on the exact release commit:

```text
exact release commit = a978187b56f97d3747f90bc10a2c068ef3f49892
workflow_dispatch Release Gate = https://github.com/atxinbao/NTPRO/actions/runs/27563266843
workflow_dispatch status = PASS
tag-triggered Release Gate = https://github.com/atxinbao/NTPRO/actions/runs/27566041100
tag-triggered status = PASS
formal tag = ntpro-rust-only-v0.7.2
formal GitHub Release = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.7.2
release name = NTPRO Rust-only v0.7.2
isDraft = false
isPrerelease = false
publishedAt = 2026-06-15T19:13:26Z
```

## Current Validation For This Post-Release PR

```text
scripts/ai/verify_fast.sh = required
git diff --check = required
text boundary scan = required
```

## Release Decision Checklist

- [x] Owner approved `ntpro-rust-only-v0.7.2` tag creation.
- [x] Exact release commit is recorded.
- [x] Hosted Release Gate passed on the exact release commit.
- [x] Tag-triggered Release Gate passed on `ntpro-rust-only-v0.7.2`.
- [x] GitHub Release body used `v0_7_2_release_notes.md` with published
      wording.
- [x] Release body keeps no orders / no real funds / no production trading
      boundary.

## Rollback Plan

Revert the V080-PRE-002 commit to restore the previous candidate readiness
wording. No runtime behavior depends on it.
