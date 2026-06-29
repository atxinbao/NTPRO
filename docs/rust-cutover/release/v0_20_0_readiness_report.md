# NTPRO v0.20.0 Production Order Lifecycle Readiness Report

Date: 2026-06-29
Executor: Codex
Milestone: `ntpro-rust-only-v0.20.0`
Status: RELEASED

## Summary

`v0.20.0` is ready for release as the Owner-Approved Production Order Lifecycle
Foundation. The readiness decision is based on the completed V200 issue chain,
local evidence files, production order lifecycle golden traces, aggregate
v20 release gates, and strict release provenance.

Plain Chinese summary: v0.20.0 的结论是“可以发布 owner-approved production order
lifecycle foundation”。它只发布受 owner approval 约束的基础 submit/readback/audit
证据链，不发布自动化实盘终端。缺审批、缺风险检查、缺 env-only signing material、
缺 readback、缺 failure/no-retry、出现 retry 或 Dashboard 操作控件，都必须阻断发布。

## Product Claim

```text
capability = Owner-Approved Production Order Lifecycle Foundation
production submit lifecycle foundation = included
owner approval = required
risk gate = required
signing material env gate = required
single submit attempt = required
post-submit response redaction = required
post-submit readback = required
failure/no-retry evidence = required
Dashboard surface = read-only audit view
golden trace coverage = included
aggregate release gate = scripts/ai/verify_release.sh v20-release-gates
strict provenance gate = scripts/ai/verify_release_strict.sh v20
Dashboard order controls = not included
implicit retry = not included
automatic cancel = not included
automatic remediation = not included
bulk order execution = not included
retry / replace / amend / flatten = not included
strategy-driven production execution = not included
general production trading platform claim = not included
```

## Completed V200 Chain

Live GitHub status captured on 2026-06-29:

```text
#611 V200-000 CLOSED
#612 V200-001 CLOSED
#613 V200-002 CLOSED
#614 V200-003 CLOSED
#615 V200-004 CLOSED
#616 V200-005 CLOSED
#617 V200-006 CLOSED
#618 V200-007 CLOSED
#619 V200-008 CLOSED
#620 V200-009 CLOSED
#621 V200-010 CLOSED
#622 V200-011 CLOSED
#623 V200-012 OPEN until this release-gate PR merges
```

The release gate requires V200-000 through V200-011 evidence documents before
publication and treats V200-012 as the release-gate/provenance closure task.

## Release-Blocking Boundary

```text
missing v0.19.1 closeout evidence = release-blocking
missing V200 evidence = release-blocking
missing owner approval = release-blocking
missing pre-submit risk gate = release-blocking
missing signing material env gate = release-blocking
missing guarded submit candidate = release-blocking
missing response redaction = release-blocking
missing post-submit readback = release-blocking
missing failure/no-retry evidence = release-blocking
missing Dashboard read-only audit = release-blocking
missing production order lifecycle golden trace = release-blocking
release manifest mismatch = release-blocking
binary provenance mismatch = release-blocking
raw credential plaintext = release-blocking
raw exchange payload persistence = release-blocking
implicit retry = release-blocking
second submit attempt = release-blocking
automatic cancel = release-blocking
automatic remediation = release-blocking
Dashboard order controls = release-blocking
Dashboard approval controls = release-blocking
Dashboard cancel controls = release-blocking
Dashboard retry controls = release-blocking
```

## Golden Trace Coverage

```text
trace = tests/golden/production_order_lifecycle_schema.jsonl
cases = 6
pre_submit_blocked_missing_approval = no submit, no retry
accepted_readback_matched_audit_closed = accepted response, matched readback, audit closed
venue_rejected_failure_no_retry = rejected response, failure evidence, no retry
unknown_response_failure_no_retry = unknown response, failure evidence, no retry
readback_mismatch_failure_no_retry = mismatch, risk visible, no retry
readback_missing_failure_no_retry = missing readback, risk visible, no retry
```

## Local Release Gate Evidence

Required local validation for the v0.20 readiness decision:

```text
scripts/ai/verify_v20_release_gates.sh
scripts/ai/verify_release.sh v20-release-gates
scripts/ai/verify_release.sh v20-strict-provenance
scripts/ai/verify_release_strict.sh v20
scripts/ai/verify_v20_order_lifecycle_golden_traces.sh
scripts/ai/run_golden_traces.sh
python3 scripts/ai/validate_golden_trace_release_scope.py --manifest docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json --trace-glob 'tests/golden/*.jsonl'
scripts/ai/check_release_surface_current.sh
scripts/ai/check_github_release_published.sh
scripts/ai/verify_fast.sh
git diff --check
```

The aggregate `v20-release-gates` stage includes:

```text
cargo test -p nautilus-risk --test v20_pre_submit_gate
cargo test -p nautilus-risk --test v20_owner_approval
cargo test -p nautilus-risk --test v20_signing_material_gate
cargo test -p nautilus-risk --test v20_submit_request_builder
cargo test -p nautilus-risk --test v20_submit_candidate
scripts/ai/verify_v20_order_lifecycle_golden_traces.sh
python3 scripts/ai/validate_golden_trace_release_scope.py --manifest docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json --trace-glob 'tests/golden/*.jsonl'
```

## Not Included

```text
product-grade live trading terminal = not included
strategy-driven production execution = not included
multi-account production execution = not included
multi-venue production execution = not included
bulk order execution = not included
automatic cancel = not included
automatic remediation = not included
retry / replace / amend / flatten = not included
Dashboard order controls = not included
Dashboard approval controls = not included
Dashboard cancel controls = not included
Dashboard retry controls = not included
raw credential or raw exchange payload publication = not included
binary asset publication = not included
general production trading platform claim = not included
```

## Final Verdict

The v0.20 source-tree package is ready for the formal
`Owner-Approved Production Order Lifecycle Foundation` release after the
`v20-release-gates` and `v20-strict-provenance` gates pass locally and in the
tag-triggered hosted release workflow.

Do not describe this release as automatic trading readiness, unattended
execution readiness, Dashboard operation-control readiness, multi-account or
multi-venue production execution readiness, or general production trading
platform readiness.
