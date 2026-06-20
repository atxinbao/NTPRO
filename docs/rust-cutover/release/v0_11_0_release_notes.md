# NTPRO Rust-only v0.11.0 Release Notes

Date: 2026-06-20
Executor: Codex
Status: READY FOR OWNER RELEASE DECISION
Tag candidate: `ntpro-rust-only-v0.11.0`
Release name candidate: `NTPRO Rust-only v0.11.0`

## Summary

`v0.11.0` is the Production Read-Only + Shadow Portfolio release candidate. It
adds release-scoped contracts and verification for reading production context
through explicit gates, recording local shadow execution/portfolio evidence,
and displaying that evidence in Dashboard as read-only status.

Plain Chinese summary: v0.11.0 是“生产只读 + 本地影子组合”的候选发布。它可以把生产
endpoint 分类清楚，可以准备公开只读和认证账户快照契约，可以生成本地 shadow intent、
shadow portfolio、生命周期和 reconciliation 证据，也可以在 Dashboard 只读展示。
它不能下生产订单，不能撤单/改单，不能动真实资金，也不能把 Dashboard 变成交易面板。

## Changed

Delivered changes:

- v0.11.0 Production Read-Only + Shadow Portfolio boundary;
- central endpoint classifier design for sandbox, production read-only, and
  forbidden production mutation surfaces;
- production public read-only probe CLI contract, offline by default;
- authenticated production account snapshot CLI contract, owner-gated,
  redacted, and offline by default;
- local shadow execution intent artifact contract;
- minimal local shadow portfolio snapshot artifact contract;
- shadow/read-only order lifecycle state model;
- production read-only reconciliation event model;
- read-only Dashboard production shadow status display;
- v0.11 offline release gate wiring for local verification, PR smoke, and
  tag-triggered release checks.

## Boundary

Included:

```text
production endpoint classification
production public read-only contract
owner-gated authenticated account snapshot contract
local shadow execution intent artifacts
local shadow portfolio snapshot artifacts
local shadow/read-only lifecycle state evidence
local reconciliation/manual-remediation event evidence
read-only Dashboard production shadow status
offline release gates
```

Not included:

```text
production order submission
production cancel, replace, amend, retry, or correction orders
real funds
production trading
automatic production reconciliation or remediation
production order lifecycle parity
Dashboard order, cancel, replace, amend, or retry controls
tag creation by this task
GitHub Release publication by this task
```

## Validation

Readiness evidence for this release package is recorded in:

```text
docs/rust-cutover/evidence/V110-000.md
docs/rust-cutover/evidence/V110-001.md
docs/rust-cutover/evidence/V110-002.md
docs/rust-cutover/evidence/V110-003.md
docs/rust-cutover/evidence/V110-004.md
docs/rust-cutover/evidence/V110-005.md
docs/rust-cutover/evidence/V110-006.md
docs/rust-cutover/evidence/V110-007.md
docs/rust-cutover/evidence/V110-008.md
docs/rust-cutover/evidence/V110-009.md
docs/rust-cutover/evidence/V110-010.md
```

Final local validation for the closure package:

```text
scripts/ai/verify_v11_offline_release_gates.sh = PASS
NTPRO_V11_LIGHTWEIGHT=1 scripts/ai/verify_v11_offline_release_gates.sh = PASS
scripts/ai/verify_release.sh v11-offline-release-gates = PASS
scripts/ai/verify_fast.sh = PASS
release-boundary grep checks = PASS
git diff --check = PASS
```

Final hosted validation before this closure slice:

```text
PR #409 = merged
merge commit = 8603e9b07b3a906d87d10f5a5ab76327daf32566
Rust Cutover Smoke / smoke = PASS
security-audit checks = PASS
```

## Release Status

This document prepares release notes for a future owner-approved formal
publication as:

```text
tag = ntpro-rust-only-v0.11.0
release name = NTPRO Rust-only v0.11.0
```

V110-010 does not create the tag and does not publish the GitHub Release.

The release boundary must continue to preserve: Production Read-Only + Shadow
Portfolio only, no production order submission, no production order mutation,
no real funds, no production trading, no automatic production remediation, and
no Dashboard order controls.
