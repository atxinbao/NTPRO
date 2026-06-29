# NTPRO Rust-only v0.19.1 Release Notes

Date: 2026-06-29
Executor: Codex
Status: CLOSEOUT IN PROGRESS - NOT PUBLISHED
Target tag: `ntpro-rust-only-v0.19.1`

## Summary

`v0.19.1` is a release-closeout and provenance hardening patch for the
published `ntpro-rust-only-v0.19.0` owner-approved single-shot actual cancel
release. It does not add a new trading capability. Its purpose is to reconcile
release evidence, publication guards, strict v19 provenance, post-merge review
attestation, and standalone v19 gate hardening before v0.20 production order
lifecycle planning starts.

Plain Chinese summary: v0.19.1 不是新能力版本。大白话：这版只补 v0.19.0 真实撤单发布后的
证据链和流程缺口，包括 PR #598 的 post-merge review attestation、v19 strict provenance、
publication guard、release closeout 和 standalone gate hardening。它不新增生产下单生命周期，
不新增自动撤单、批量撤单、retry、二次撤单或 Dashboard 撤单按钮。

## Included Closeout Evidence

```text
V191-001 = v0.19.0 release closeout evidence
V191-002 = v0.18.1 prerequisite release evidence reconciliation
V191-003 = current release surface alignment to v0.19.0
V191-004 = v0.19.0 release publication guard
V191-005 = v19 strict release provenance
V191-006 = V190-004 / PR #598 post-merge review attestation
V191-007 = standalone v19 release gate hardening
```

## V191-006 Review Attestation

`docs/rust-cutover/evidence/V191-006_actual_cancel_review_attestation.md`
records the V190-004 / PR #598 post-merge review attestation. PR #598 added the
`production-mutation-actual-cancel-single-shot` command and was marked
`REVIEW_REQUIRED`, but live GitHub review submissions were absent
(`reviews=[]`, empty `reviewDecision`). The V191-006 evidence records the
compensating post-merge review and explicitly states that it does not replace
the missing pre-merge GitHub review submission.

The attestation covers:

```text
PR #598 and issue #581
post-merge review identity and date
reviewed command surface
production-mutation-actual-cancel-single-shot boundary
manual-online env gates
owner approval consumption before/after attempt
one order / one venue / one attempt
no retry
no bulk
no second cancel
no Dashboard execution
readback-required follow-up
failure-evidence follow-up
known limitation: post-merge review is not a pre-merge review replacement
```

## Not Included

```text
production order submit lifecycle = not included
automatic cancel = not included
automatic remediation = not included
bulk cancel = not included
cancel all open orders = not included
retry / replace / amend / flatten = not included
second cancel = not included
Dashboard cancel button = not included
Dashboard approval button = not included
Dashboard credential input = not included
release tag publication = pending owner release decision
```

## Validation

Required local validation for this closeout note:

```text
gh pr view 598 --repo atxinbao/NTPRO --json number,title,state,mergedAt,reviews,reviewDecision,url
rg -n "V190-004|PR #598|post-merge review|REVIEW_REQUIRED|owner approval consumption|no retry|no bulk|no second cancel|Dashboard" docs/rust-cutover/evidence docs/rust-cutover/release
git diff --check
scripts/ai/verify_fast.sh
```

## Release Status

```text
v0.19.1 status = closeout in progress
GitHub Release = not published
tag = not created
current formal release = ntpro-rust-only-v0.19.0
next capability track = v0.20.0 after all v0.19.1 blockers close
```
