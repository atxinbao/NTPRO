# NTPRO Rust-only v0.11.1 Release Notes

Date: 2026-06-20
Executor: Codex
Status: READY FOR OWNER RELEASE DECISION
Candidate tag: `ntpro-rust-only-v0.11.1`

## Summary

`v0.11.1` is a hardening patch for the published `v0.11.0` Production
Read-Only Contract + Offline Shadow Portfolio release. It closes wording,
gate, artifact-health, endpoint-classification, and field-semantics gaps without
expanding the product capability.

Plain Chinese summary: v0.11.1 是 v0.11.0 的补丁，不是新交易能力。它把“离线合约、
只读、shadow、本地证据”的边界写得更清楚，门禁跑得更完整，Dashboard 对坏 artifact
更敏感。它不代表已经可以线上读取生产账户，也不代表可以实盘下单。

## Changed

- Aligned v0.11 wording so contract/offline evidence cannot be mistaken for
  successful online production reads.
- Added the central Rust endpoint classifier API and deny tests for production
  mutation and order-state surfaces.
- Added the production shadow manifest contract.
- Hardened Dashboard production shadow health so missing, malformed, schema
  mismatched, or boundary-violating artifacts degrade status.
- Wired the public read probe into v11 offline release gates.
- Clarified `/api/v3/openOrders`, `/api/v3/allOrders`, and `/api/v3/order`
  order-state reads are not part of v0.11 account snapshot support.
- Added explicit artifact fields:

```text
contract_ready
online_read_allowed
```

Existing `read_allowed` remains as a backward-compatible local contract
readiness signal; it is not an online production read permission.

## Boundary

Included:

```text
release-surface wording hardening
endpoint classifier API and deny tests
public read/account snapshot gate coverage
Dashboard production shadow health hardening
manifest and artifact contract clarity
field-semantics clarity for offline contracts
```

Not included:

```text
production online read runtime
successful online production public/account reads
production open-order or order-state reads
production order submission
production cancel, replace, amend, retry, or correction orders
real funds
production trading
automatic production reconciliation or remediation
Dashboard order/cancel/replace/amend controls
```

## Validation

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V111-001.md
docs/rust-cutover/evidence/V111-002.md
docs/rust-cutover/evidence/V111-003.md
docs/rust-cutover/evidence/V111-004.md
docs/rust-cutover/evidence/V111-005.md
docs/rust-cutover/evidence/V111-006.md
docs/rust-cutover/evidence/V111-007.md
docs/rust-cutover/evidence/V111-008.md
```

Local validation for this release-note package:

```text
NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG=1 scripts/ai/check_release_surface_current.sh = PASS
scripts/ai/verify_release.sh v11-offline-release-gates = PASS
scripts/ai/verify_fast.sh = PASS
git diff --check = PASS
```

## Release Status

This document is release-note material for a possible owner-approved
`ntpro-rust-only-v0.11.1` GitHub Release. This PR does not create the tag and
does not publish the GitHub Release.

The release boundary must continue to preserve: Production Read-Only Contract
and Offline Shadow Portfolio hardening only, no successful online production
read claim, no production order submission, no production order mutation, no
real funds, no production trading, no automatic production remediation, and no
Dashboard order controls.
