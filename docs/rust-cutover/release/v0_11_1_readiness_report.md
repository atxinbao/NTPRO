# NTPRO v0.11.1 Production Read-Only Contract Hardening Readiness Report

Date: 2026-06-20
Executor: Codex
Milestone: `v0.11.1`
Status: READY FOR OWNER RELEASE DECISION

## Summary

`v0.11.1` is a patch/hardening queue for the published `v0.11.0` Production
Read-Only Contract + Offline Shadow Portfolio line. It closes release-surface,
gate-wiring, artifact-health, and field-semantics gaps found after v0.11.0.

Plain Chinese summary: v0.11.1 不是新能力版本。它只是把 v0.11.0 的公开文档、
发布门禁、Dashboard artifact 健康判断、endpoint 分类和 JSON 字段语义补严。它不打开
生产网络读取，不下生产订单，不碰真实资金，也不让 Dashboard 变成下单面板。

## Product Claim

```text
capability = v0.11.0 Production Read-Only Contract + Offline Shadow Portfolio hardening
current published release before v0.11.1 = ntpro-rust-only-v0.11.0
candidate patch version = v0.11.1
default execution posture = offline fail-closed
production public read-only contract = hardened, explicitly gated, offline by default
production authenticated account snapshot contract = hardened, owner-gated, redacted, offline by default
successful online production reads = not included
production open-order/order-state reads = not included
production order submission = not included
production order mutation = not included
real funds = not included
production trading = not included
Dashboard order controls = not included
tag creation = not performed by this readiness PR
GitHub Release publication = not performed by this readiness PR
```

## Included

```text
v0.11 release wording aligned to contract/offline reality
central endpoint classifier Rust API and deny tests
production shadow manifest contract
Dashboard production shadow artifact health hardening
public read probe wired into v11 offline release gates
openOrders/order-state wording corrected as out of scope
read_allowed field semantics normalized with contract_ready and online_read_allowed
v0.11.1 readiness and release-note material
```

## Not Included

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
new tag creation
GitHub Release publication
```

## Task Accounting

| Task | Status | Evidence | PR | Notes |
| --- | --- | --- | --- | --- |
| V111-001 | PASS | `docs/rust-cutover/evidence/V111-001.md` | #412 | Aligns v0.11 wording to contract/offline reality. |
| V111-002 | PASS | `docs/rust-cutover/evidence/V111-002.md` | #413 | Adds central endpoint classifier API and deny tests. |
| V111-003 | PASS | `docs/rust-cutover/evidence/V111-003.md` | #414 | Adds production shadow manifest contract. |
| V111-004 | PASS | `docs/rust-cutover/evidence/V111-004.md` | #415 | Hardens Dashboard production shadow artifact health. |
| V111-005 | PASS | `docs/rust-cutover/evidence/V111-005.md` | #416 | Wires public read probe into v11 offline release gates. |
| V111-006 | PASS | `docs/rust-cutover/evidence/V111-006.md` | #417 | Clarifies `/api/v3/openOrders` and order-state reads are out of scope. |
| V111-007 | PASS | `docs/rust-cutover/evidence/V111-007.md` | #418 | Adds explicit `contract_ready` / `online_read_allowed` artifact semantics. |
| V111-008 | PASS CANDIDATE | `docs/rust-cutover/evidence/V111-008.md` | this PR | Prepares v0.11.1 readiness and release notes. |

## Gate Evidence

Required local validation for the v0.11.1 readiness material:

```bash
NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG=1 scripts/ai/check_release_surface_current.sh
scripts/ai/verify_release.sh v11-offline-release-gates
scripts/ai/verify_fast.sh
git diff --check
```

The v11 offline release gate verifies:

```text
production public read-only probe contract
authenticated account snapshot contract
Dashboard read-only production shadow status
release boundary markers
network_attempted=false
production_orders_submitted=0
production_order_mutations_attempted=0
dashboard_order_controls=false
```

## Release Decision

This report is readiness material only. It does not create a tag and does not
publish a GitHub Release.

Before publishing `ntpro-rust-only-v0.11.1`, the owner should verify:

```text
main includes all V111-001 through V111-008 PRs
GitHub hosted Rust Cutover Smoke is PASS on the release candidate commit
README / ROADMAP / versioning / release notes agree on hardening-only scope
GitHub open PR count is acceptable for release closure
worktree is clean
```

## Final Verdict

`v0.11.1` is ready for owner release decision as a hardening-only patch after
V111-001 through V111-008 pass and merge.

Do not describe this readiness PASS as successful online production read
readiness, production order submission readiness, real-funds readiness,
production trading readiness, automatic production reconciliation readiness, or
Dashboard order-control readiness.
