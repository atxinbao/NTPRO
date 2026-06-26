# NTPRO v0.18.0 Owner-Approved Cancel Recovery Preview Readiness Report

Date: 2026-06-26
Executor: Codex
Milestone: `v0.18.0`
Status: PASS - readiness docs prepared, release tag not published

## Summary

`v0.18.0` packages the owner-approved cancel recovery preview and gate evidence
line. It starts from the v0.17 orphan-risk evidence, prepares one cancel
candidate preview, checks the candidate with a fail-closed risk gate, records a
manual owner approval lifecycle, defines cancel response redaction and
post-cancel readback contracts, ties the chain into incident/audit closeout,
shows the state in a read-only Dashboard panel, and adds aggregate release
gates.

Plain Chinese summary: v0.18.0 是“撤单恢复预览和门禁”版本。大白话：系统可以把要撤哪一笔、
风险门禁、Owner 人工审批、响应脱敏、撤单后回查、审计 closeout 和 Dashboard 只读展示都准备好；
但这版不是真实撤单版本，不会调用交易所撤单接口，不会自动撤单，也没有 Dashboard 撤单按钮。

## Product Claim

```text
capability = Owner-Approved Cancel Recovery Preview
capability_expansion = preview_gate_approval_only
release tag = not published by V180-011
release name = NTPRO Rust-only v0.18.0
default execution posture = offline fail-closed
lineage_scope = single_v16_mutation_candidate
cancel request preview = included
cancel risk gate = included
manual owner approval lifecycle = included
cancel response redaction contract = included
post-cancel readback contract = included
incident and audit closeout contract = included
Dashboard surface = read-only cancel recovery evidence only
aggregate release gate = included
actual cancel send = not included
automatic cancel = disabled
automatic remediation = disabled
Dashboard order controls = disabled
Dashboard cancel controls = disabled
retry_attempted = false
cancel_attempted = false
remediation_attempted = false
production_order_mutation_allowed = false
network_cancel_endpoint_attempted = false
```

## Included

```text
v0.18 owner-approved cancel recovery scope decision
v0.18 artifact contract definitions
cancel request preview artifact
cancel risk gate artifact
manual owner approval lifecycle artifact
cancel response redaction contract
post-cancel readback contract
incident/audit closeout contract
read-only Dashboard cancel recovery panel
v0.18 aggregate release gates
v0.18 readiness report and release notes
```

## Not Included

```text
actual cancel send
DELETE /api/v3/order
DELETE /api/v3/openOrders
automatic cancel
strategy-driven cancel
cancel all open orders
bulk cancel
multi-account cancel recovery
multi-venue cancel recovery
retry, replace, amend, flatten, or remediation action
automatic remediation
Dashboard cancel button
Dashboard order controls
Dashboard credential input
raw secret persistence
raw exchange response persistence
production trading platform claim
release tag publication
```

## Merged PR Accounting

| Task | Issue | PR | Merged at | Merge commit | Evidence | Capability expansion |
| --- | --- | --- | --- | --- | --- | --- |
| V180-001 | #539 | #559 | 2026-06-26T12:36:00Z | `11d3c207ffd7d11b554934c58816afa696e8798c` | `docs/rust-cutover/evidence/V180-001.md` | Scope decision only |
| V180-002 | #540 | #560 | 2026-06-26T12:43:20Z | `c9a0b10a6a58200f8f796f77f532a1d754d5306b` | `docs/rust-cutover/evidence/V180-002.md` | Artifact contracts only |
| V180-003 | #541 | #561 | 2026-06-26T13:27:36Z | `edcba303f9b708987e3c5a2d1c7c009ed467d03d` | `docs/rust-cutover/evidence/V180-003.md` | Adds one-candidate preview evidence |
| V180-004 | #542 | #562 | 2026-06-26T14:20:28Z | `21dcb3afe3229307348ea0ef04cf78fa4949084d` | `docs/rust-cutover/evidence/V180-004.md` | Adds fail-closed risk gate evidence |
| V180-005 | #543 | #563 | 2026-06-26T14:59:59Z | `a13d8e03d92ff830f279853127cfcccc402e5f58` | `docs/rust-cutover/evidence/V180-005.md` | Adds manual owner approval lifecycle evidence |
| V180-006 | #544 | #564 | 2026-06-26T15:40:31Z | `c0578ef131ab8a39c683b0dface535b22c85daae` | `docs/rust-cutover/evidence/V180-006.md` | Adds redacted response contract evidence |
| V180-007 | #545 | #565 | 2026-06-26T16:20:30Z | `ea10d72496a0aa0300559c485c84dc936d98e7d9` | `docs/rust-cutover/evidence/V180-007.md` | Adds post-cancel readback contract evidence |
| V180-008 | #546 | #566 | 2026-06-26T17:06:03Z | `844eeae20578783f25d5c7c7224cebef8a605856` | `docs/rust-cutover/evidence/V180-008.md` | Adds incident/audit closeout evidence |
| V180-009 | #547 | #567 | 2026-06-26T17:49:20Z | `dd3d4441975ec12f7e49426f0c2947577e26b355` | `docs/rust-cutover/evidence/V180-009.md` | Adds read-only Dashboard visibility only |
| V180-010 | #548 | #568 | 2026-06-26T18:30:21Z | `68c0df4985767fc879cc56858926abec9f4d68aa` | `docs/rust-cutover/evidence/V180-010.md` | Adds aggregate release gates only |
| V180-011 | #549 | IN THIS SOURCE TREE | pending before PR | pending before PR | `docs/rust-cutover/evidence/V180-011.md` | Adds readiness and release-note accounting only |

## Hosted Gate Evidence

Hosted PR smoke evidence for the v0.18 chain:

```text
V180-001 PR #559 smoke SUCCESS, run 28238327840, job 83658523981
V180-002 PR #560 smoke SUCCESS, run 28238646053, job 83659613740
V180-003 PR #561 smoke SUCCESS, run 28239677466, job 83663116026
V180-004 PR #562 smoke SUCCESS, run 28242518009, job 83672795563
V180-005 PR #563 smoke SUCCESS, run 28244710460, job 83680503409
V180-006 PR #564 smoke SUCCESS, run 28247089663, job 83688827205
V180-007 PR #565 smoke SUCCESS, run 28249293792, job 83696428771
V180-008 PR #566 smoke SUCCESS, run 28251759505, job 83704754746
V180-009 PR #567 smoke SUCCESS, run 28254034782, job 83712457887
V180-010 PR #568 smoke SUCCESS, run 28256269452, job 83719995949
```

## Local Release Gate Evidence

Required local validation for the v0.18 readiness decision:

```text
scripts/ai/verify_release.sh v18-release-gates
scripts/ai/verify_fast.sh
git diff --check
```

The aggregate `v18-release-gates` stage includes:

```text
scripts/ai/verify_v18_cancel_request_preview.sh
scripts/ai/verify_v18_cancel_risk_gate.sh
scripts/ai/verify_v18_manual_owner_approval_lifecycle.sh
scripts/ai/verify_v18_cancel_response_redaction.sh
scripts/ai/verify_v18_post_cancel_readback.sh
scripts/ai/verify_v18_cancel_recovery_incident_audit_closeout.sh
scripts/ai/verify_v18_dashboard_cancel_recovery_panel.sh
```

## Default Fail-Closed Proof

The v0.18 release line requires the following default markers:

```text
default_offline = true
actual_cancel_send_allowed = false
cancel_attempted = false
cancel_requests_sent = 0
network_attempted = false
network_cancel_endpoint_attempted = false
network_readback_endpoint_attempted = false
readback_execution_attempted = false
production_order_mutation_allowed = false
production_order_mutations_attempted = 0
retry_attempted = false
replace_attempted = false
amend_attempted = false
flatten_attempted = false
remediation_attempted = false
automatic_cancel_allowed = false
automatic_remediation_allowed = false
dashboard_order_controls_enabled = false
dashboard_cancel_controls_enabled = false
api_key_value_recorded = false
api_secret_value_recorded = false
signature_recorded = false
signed_query_recorded = false
signed_url_recorded = false
raw_exchange_response_recorded = false
raw_readback_body_recorded = false
```

## Next-Version Boundary

Actual single-shot cancel remains a v0.19+ scope decision. A later release must
explicitly define and gate any exchange cancel endpoint call, signing material
use, approval consumption, kill switch, idempotency, raw response redaction,
post-send readback, retry prohibition, rollback, and Dashboard boundary before
any real cancel can be considered.

## Release Closure Status

```text
latest formal release before this line = ntpro-rust-only-v0.17.1
v0.18.0 readiness = PASS - docs prepared
v0.18.0 tag = not published by V180-011
v0.18.0 GitHub Release = not published by V180-011
```

## Final Verdict

The v0.18 source-tree package is ready as `Owner-Approved Cancel Recovery
Preview` evidence. Do not describe this readiness as actual cancel execution
readiness, automatic remediation readiness, strategy-driven cancel readiness,
multi-account/multi-venue cancel readiness, Dashboard cancel-control readiness,
or general production trading readiness.
