# NTPRO Rust-only v0.7.2 Release Notes

Date: 2026-06-15
Executor: Codex
Release line: v0.7.2
Status: Released

## Release Identity

```text
Source tag: ntpro-rust-only-v0.7.2
Capability: v0.7 read-only Binance testnet connectivity proof wording/evidence patch
Boundary: release surface, artifact note accuracy, and release-closure evidence only
Publication: formal GitHub Release after owner approval and hosted gates passed
```

## Plain Chinese Summary

`v0.7.2` 是 `v0.7.1` 之后的发布口径和证据补丁，不是新交易能力版本。

它主要做三件事：

- 把 v0.7.1 发布后的 README、ROADMAP、release notes/readiness wording 收口；
- 修正 explicit opt-in HTTP 只读探测 artifact notes，避免已经尝试联网时还写“没有打开 socket”；
- 把 v0.7.1 和 v0.7.2 的 hosted gate、tag-triggered gate、GitHub Release URL
  记录清楚。

## Included

- v0.7.1 post-release wording finalized.
- v0.7.1 historical release-surface wording finalized.
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
- v0.7.2 release-closure evidence recorded:
  - exact release commit;
  - workflow_dispatch Release Gate;
  - tag-triggered Release Gate;
  - formal GitHub Release URL;
  - publication flags.
- README and ROADMAP current release wording aligned to
  `ntpro-rust-only-v0.7.2`.

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

## Included PRs

```text
V072-001 / #326: v0.7.1 post-release wording
V072-002 / #327: README and ROADMAP current release alignment
V072-003 / #328: online HTTP read-only boundary notes
V072-004 / #329: v0.7.1 release-closure evidence
V072-005 / #330: v0.7.2 release notes and readiness closeout
V080-PRE-001 / #331: v0.7.2 release notes published wording
V080-PRE-002 / #332: v0.7.2 readiness released wording
V080-PRE-003 / #333: README and ROADMAP v0.7.2 current release alignment
V080-PRE-004: v0.7.2 release-closure evidence
```

## Validation Evidence

The v0.7.2 release was validated with:

```bash
scripts/ai/verify_fast.sh
scripts/ai/verify_v07_default_offline_gate.sh
scripts/ai/verify_v07_manual_online_gate.sh
scripts/ai/verify_release.sh v07-default-offline-gate v07-manual-online-preflight
git diff --check
```

The exact release commit was used for both hosted release gates before
publishing the formal GitHub Release.

## Published Release Boundary

`ntpro-rust-only-v0.7.2` is the formal GitHub Release for this
wording/evidence patch. The release remains wording/evidence only: no order
submission, no authenticated account access, no real funds, no production
Binance connectivity, and no production trading.

## Release Closure Evidence

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
detailed evidence = docs/rust-cutover/evidence/V080-PRE-004.md
```
