# NTPRO Rust-only v0.17.0 Release Notes

Date: 2026-06-24
Executor: Codex
Status: READY FOR OWNER RELEASE DECISION
Tag: `ntpro-rust-only-v0.17.0`
Release name: `NTPRO Rust-only v0.17.0`
Release URL: pending owner publication

## Summary

`v0.17.0` is the NTPRO production reconciliation and orphan recovery evidence
release. It extends the v0.16 single owner-approved production mutation
candidate with local/offline evidence for ledger persistence, readback mapping,
reconciliation classification, orphan order risk, restart recovery, Dashboard
visibility, and failure incident semantics.

Plain Chinese summary: v0.17.0 是“对账和孤儿单风险证据”版本。大白话：它帮助交易系统在
一笔生产候选订单之后，看清楚本地和交易所状态是否对得上、有没有孤儿单风险、失败原因是
什么。它默认不联网、不下单、不撤单，也没有 Dashboard 交易按钮。

## Changed

- Added a local production order ledger for the v0.16 single lineage.
- Added a redacted exchange readback mapper for order/openOrders evidence.
- Added a local-vs-exchange reconciliation classifier.
- Added orphan order risk detection and new-order blocking evidence.
- Documented the owner-approved cancel recovery boundary without enabling
  cancel execution.
- Added restart recovery evidence from an existing local ledger.
- Added a read-only Dashboard panel for reconciliation and orphan risk.
- Integrated v0.16 failure semantics into v0.17 incident outcomes.
- Added aggregate v0.17 release gates.
- Added v0.17 readiness and release-note material.

## Boundary

Included:

```text
Production Reconciliation And Orphan Recovery Evidence
capability_expansion_from_v16 = reconciliation_evidence_only
lineage_scope = single_v16_mutation_candidate
default execution posture = offline fail-closed
local ledger
redacted readback mapper
reconciliation classifier
orphan order detector
restart recovery evidence
failure incident semantics
read-only Dashboard evidence
aggregate v0.17 release gate
```

Not included:

```text
network readback execution = not included
production order submission = not included
production order mutation = not included
actual cancel send = deferred
automatic cancel = disabled
Dashboard order controls = disabled
Dashboard cancel controls = disabled
retry_attempted = false
cancel_attempted = false
remediation_attempted = false
strategy-driven production execution
multi-account production execution
multi-venue production execution
real-funds proof in CI
general production trading platform claim
```

## Merged PR Accounting

| PR | Classification | Included in capability claim | Notes |
| --- | --- | --- | --- |
| #518 | V170-000 | Yes, boundary only | Defines the v0.17 reconciliation/orphan recovery scope. |
| #519 | V170-001 | Yes, evidence only | Adds the restart-readable local order ledger. |
| #520 | V170-002 | Yes, evidence only | Maps redacted exchange readback fixtures into normalized state. |
| #521 | V170-003 | Yes, evidence only | Classifies local-vs-exchange state outcomes. |
| #522 | V170-004 | Yes, evidence only | Detects orphan risk and blocks new orders in evidence. |
| #523 | V170-005 | No active cancel capability | Documents future cancel boundary only. |
| #524 | V170-006 | Yes, evidence only | Proves restart can resume from local ledger evidence. |
| #525 | V170-007 | No Dashboard control expansion | Adds read-only Dashboard visibility. |
| #526 | V170-008 | Yes, incident evidence only | Maps failure modes into incident outcomes without remediation. |
| V170-009 | Release accounting | No product-surface expansion | Adds aggregate gates, readiness, and release notes. |

## Validation

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V170-000.md
docs/rust-cutover/evidence/V170-001.md
docs/rust-cutover/evidence/V170-002.md
docs/rust-cutover/evidence/V170-003.md
docs/rust-cutover/evidence/V170-004.md
docs/rust-cutover/evidence/V170-005.md
docs/rust-cutover/evidence/V170-006.md
docs/rust-cutover/evidence/V170-007.md
docs/rust-cutover/evidence/V170-008.md
docs/rust-cutover/evidence/V170-009.md
```

Required release validation for this package:

```text
scripts/ai/verify_release.sh v17-release-gates
scripts/ai/verify_release.sh release-surface-current-guard
scripts/ai/verify_fast.sh
git diff --check
hosted Rust Cutover Smoke
hosted security-audit when audit-relevant paths change
```

## Release Status

This document is prepared as the GitHub Release note body for:

```text
tag = ntpro-rust-only-v0.17.0
release name = NTPRO Rust-only v0.17.0
release URL = pending owner publication
```

The release boundary must continue to preserve: one v0.16 single lineage only,
default offline fail-closed execution, no new production order submission, no
production order mutation, no network readback execution, no actual cancel
send, no retry/replace/amend/correction/flatten/remediation, no automatic
orphan cleanup, no Dashboard order/cancel controls, no real-funds claim, and no
general production trading platform claim.
