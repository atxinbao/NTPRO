# NTPRO Rust-only v0.18.0 Release Notes

Date: 2026-06-26
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.18.0`
Release name: `NTPRO Rust-only v0.18.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.18.0`
Release commit: `6790688ae46d1b25806f3d1d25146c9b47d43328`
Published at: `2026-06-27T07:58:22Z`
Hosted release gate: `https://github.com/atxinbao/NTPRO/actions/runs/28281346239`

## Summary

`v0.18.0` is the owner-approved cancel recovery preview and gate release. It
extends the v0.17 reconciliation/orphan-risk evidence line by preparing a
single cancel candidate preview, proving a fail-closed risk gate, recording
manual owner approval lifecycle evidence, defining redacted response and
post-cancel readback contracts, closing the chain with incident/audit evidence,
showing the state in a read-only Dashboard panel, and adding aggregate release
gates.

Plain Chinese summary: v0.18.0 是“撤单恢复预览和门禁”版本。大白话：这版能准备撤单证据链，
让 Owner 看见候选订单、风险门禁、审批状态、脱敏和审计结果；但它不是真实撤单版本，不会发
撤单请求，不会自动补救，也没有 Dashboard 撤单按钮。

## Changed

- Added the v0.18 cancel recovery preview scope decision.
- Added versioned cancel recovery artifact contracts.
- Added the cancel request preview artifact.
- Added the cancel risk gate artifact.
- Added the manual owner approval lifecycle artifact.
- Added the cancel response redaction contract.
- Added the post-cancel readback contract.
- Added the incident/audit closeout contract.
- Added a read-only Dashboard cancel recovery panel.
- Added aggregate v0.18 release gates.
- Added v0.18 readiness and release-note material.

## Boundary

Included:

```text
Owner-Approved Cancel Recovery Preview
capability_expansion = preview_gate_approval_only
lineage_scope = single_v16_mutation_candidate
cancel request preview
cancel risk gate
manual owner approval lifecycle
cancel response redaction contract
post-cancel readback contract
incident/audit closeout contract
read-only Dashboard evidence
aggregate v0.18 release gate
```

Not included:

```text
actual cancel send = not included
automatic cancel = disabled
automatic remediation = disabled
Dashboard order controls = disabled
Dashboard cancel controls = disabled
retry_attempted = false
cancel_attempted = false
remediation_attempted = false
DELETE /api/v3/order
DELETE /api/v3/openOrders
strategy-driven cancel
cancel all open orders
bulk cancel
multi-account cancel recovery
multi-venue cancel recovery
production trading claim
```

## Merged PR Accounting

| PR | Classification | Included in capability claim | Notes |
| --- | --- | --- | --- |
| #559 | V180-001 | Yes, boundary only | Defines owner-approved cancel recovery preview scope. |
| #560 | V180-002 | Yes, contracts only | Defines versioned artifact schemas and boundary fields. |
| #561 | V180-003 | Yes, evidence only | Adds one-candidate cancel request preview. |
| #562 | V180-004 | Yes, evidence only | Adds fail-closed cancel risk gate. |
| #563 | V180-005 | Yes, evidence only | Adds manual owner approval lifecycle. |
| #564 | V180-006 | Yes, evidence only | Adds redacted cancel response contract. |
| #565 | V180-007 | Yes, evidence only | Adds post-cancel readback contract. |
| #566 | V180-008 | Yes, evidence only | Adds incident/audit closeout contract. |
| #567 | V180-009 | No Dashboard control expansion | Adds read-only Dashboard visibility. |
| #568 | V180-010 | No product-surface expansion | Adds aggregate release gates and hosted stages. |
| V180-011 | Release accounting | No product-surface expansion | Adds readiness and release notes. |

## Validation

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V180-001.md
docs/rust-cutover/evidence/V180-002.md
docs/rust-cutover/evidence/V180-003.md
docs/rust-cutover/evidence/V180-004.md
docs/rust-cutover/evidence/V180-005.md
docs/rust-cutover/evidence/V180-006.md
docs/rust-cutover/evidence/V180-007.md
docs/rust-cutover/evidence/V180-008.md
docs/rust-cutover/evidence/V180-009.md
docs/rust-cutover/evidence/V180-010.md
docs/rust-cutover/evidence/V180-011.md
```

Required release validation for this package:

```text
scripts/ai/verify_release.sh v18-release-gates
scripts/ai/verify_fast.sh
git diff --check
hosted Rust Cutover Smoke
hosted Rust Cutover Release Gate run 28281346239, 50 jobs, 0 failures
hosted security-audit when audit-relevant paths change
```

Hosted v0.18 PR smoke evidence:

```text
PR #559 smoke SUCCESS, run 28238327840, job 83658523981
PR #560 smoke SUCCESS, run 28238646053, job 83659613740
PR #561 smoke SUCCESS, run 28239677466, job 83663116026
PR #562 smoke SUCCESS, run 28242518009, job 83672795563
PR #563 smoke SUCCESS, run 28244710460, job 83680503409
PR #564 smoke SUCCESS, run 28247089663, job 83688827205
PR #565 smoke SUCCESS, run 28249293792, job 83696428771
PR #566 smoke SUCCESS, run 28251759505, job 83704754746
PR #567 smoke SUCCESS, run 28254034782, job 83712457887
PR #568 smoke SUCCESS, run 28256269452, job 83719995949
PR #569 smoke SUCCESS, run 28257950925, job 83725730229
```

## Release Status

This document is published as the GitHub Release note body for:

```text
tag = ntpro-rust-only-v0.18.0
release name = NTPRO Rust-only v0.18.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.18.0
release commit = 6790688ae46d1b25806f3d1d25146c9b47d43328
published at = 2026-06-27T07:58:22Z
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28281346239
hosted release gate result = success, 50 jobs, 0 failures
```

The release boundary must continue to preserve: one v0.16 single lineage only,
default offline fail-closed execution, no real cancel send, no automatic cancel,
no retry/replace/amend/flatten/remediation, no raw secret or raw response
persistence, no Dashboard order/cancel controls, no multi-account or multi-venue
cancel recovery, no real-funds proof, and no general production trading platform
claim.

## Next Version

Actual single-shot cancel remains a v0.19+ scope decision. v0.18.0 does not
authorize exchange cancel endpoint execution.
