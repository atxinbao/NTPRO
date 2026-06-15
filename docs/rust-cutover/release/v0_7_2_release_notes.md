# NTPRO Rust-only v0.7.2 Release Notes

Date: 2026-06-15
Executor: Codex
Release line: v0.7.2
Status: Release candidate only - not tagged, not published

## Release Identity

```text
Candidate tag: ntpro-rust-only-v0.7.2
Capability: v0.7 read-only Binance testnet connectivity proof wording/evidence patch
Boundary: release surface, artifact note accuracy, and release-closure evidence only
Publication: requires separate owner approval
```

## Plain Chinese Summary

`v0.7.2` 是 `v0.7.1` 之后的发布口径和证据补丁，不是新交易能力版本。

它主要做三件事：

- 把 v0.7.1 发布后的 README、ROADMAP、release notes/readiness wording 收口；
- 修正 explicit opt-in HTTP 只读探测 artifact notes，避免已经尝试联网时还写“没有打开 socket”；
- 把 v0.7.1 正式发布的 hosted gate、tag-triggered gate、GitHub Release URL 记录清楚。

## Included

- v0.7.1 post-release wording finalized.
- README and ROADMAP current release wording aligned to
  `ntpro-rust-only-v0.7.1`.
- Manual online HTTP read-only boundary notes corrected:
  - offline/blocked artifacts may still say no socket is opened;
  - explicit opt-in HTTP read-only attempts state that a Binance testnet public
    HTTP read-only socket was opened;
  - credential values remain unrecorded;
  - no orders, no real funds, no production trading.
- v0.7.1 release-closure evidence recorded:
  - workflow_dispatch Release Gate;
  - tag-triggered Release Gate;
  - formal GitHub Release URL.
- v0.7.2 readiness package prepared.

## Not Included

`v0.7.2` does not claim these capabilities:

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

## Candidate PRs

```text
V072-001 / #326: v0.7.1 post-release wording
V072-002 / #327: README and ROADMAP current release alignment
V072-003 / #328: online HTTP read-only boundary notes
V072-004 / #329: v0.7.1 release-closure evidence
V072-005: v0.7.2 release notes and readiness closeout
```

## Validation Required Before Publication

Before any formal `ntpro-rust-only-v0.7.2` tag or GitHub Release:

```bash
scripts/ai/verify_fast.sh
scripts/ai/verify_v07_default_offline_gate.sh
scripts/ai/verify_v07_manual_online_gate.sh
scripts/ai/verify_release.sh v07-default-offline-gate v07-manual-online-preflight
git diff --check
```

The exact post-merge release commit must be used for hosted release-gate
evidence. A GitHub Release must not be published until the owner explicitly
approves the tag/release step.

## Published Release Boundary

This file is a candidate release note. Until the owner approves a tag and
GitHub Release, `ntpro-rust-only-v0.7.1` remains the current published release.

If v0.7.2 is published, the release remains wording/evidence only: no order
submission, no authenticated account access, no real funds, no production
Binance connectivity, and no production trading.
