# v0.19.0 Owner Approval Execution Lifecycle

Date: 2026-06-27
Executor: Codex
Milestone: `v0.19.0`
Task: `V190-003`
Status: IMPLEMENTATION CONTRACT

## Summary

This document defines the v0.19.0 owner approval execution lifecycle for the
owner-approved single-shot actual cancel line. It upgrades the v0.18 preview
approval artifact into a v0.19 authorization artifact that can authorize one
future actual cancel attempt only when it is bound to the same order evidence,
risk gate, venue, owner identity, timestamp, reason, safety contract, and
release provenance.

Plain Chinese summary: 这次把 v0.18 的“预览审批”升级成 v0.19 的“真撤单授权证据”。大白话：
approval 只有在 owner 手工批准、未过期、未使用、绑定同一个订单、同一个 venue、同一个 risk gate 和
同一个 release provenance 时，才会授权后续一个真实撤单尝试；它本身仍不发送撤单、不联网、不加
Dashboard 按钮。

## CLI Surface

```text
nautilus live production-mutation-actual-cancel-owner-approval-lifecycle \
  --run-id v190-owner-approval-lifecycle \
  --actual-cancel-safety-contract docs/rust-cutover/release/v0_19_0_actual_cancel_safety_contract.md \
  --release-manifest docs/rust-cutover/release/v0_18_1_release_manifest.json \
  --cancel-risk-gate runs/v180/cancel-risk-gate.json \
  --expected-order-lineage-id lineage-v160-single-shot \
  --expected-symbol BTCUSDT \
  --expected-account-label prod-account-redacted \
  --venue binance_spot \
  --expected-release-tag ntpro-rust-only-v0.18.1 \
  --approval-state approved \
  --manual-approval-id owner-approval-v190-003 \
  --approved-by owner \
  --approval-reason orphan-risk-single-order-cancel \
  --now-unix-ms 1718400000000 \
  --expires-at-unix-ms 1718400060000 \
  --output runs/v190/actual-cancel-owner-approval-lifecycle.json \
  --allow-production-mutation-actual-cancel-owner-approval-lifecycle \
  --confirm-actual-cancel-safety-contract \
  --confirm-one-order-one-venue-one-attempt \
  --confirm-single-use-approval \
  --confirm-approval-expiry \
  --confirm-bind-order-risk-gate-release-provenance \
  --confirm-audit-evidence \
  --confirm-no-dashboard-approval \
  --confirm-no-automatic-cancel \
  --confirm-no-bulk-cancel \
  --confirm-no-retry \
  --confirm-no-submit-lifecycle \
  --confirm-no-network \
  --confirm-no-secret-persistence
```

## Lifecycle States

```text
created  = lifecycle exists but does not authorize send
approved = authorizes exactly one future actual cancel attempt if all evidence matches
expired  = fail closed
used     = fail closed and records consumed/audit evidence
rejected = fail closed and records rejection audit evidence
audited  = fail closed because lifecycle is already post-decision audit evidence
```

Only `approved` can set `approval_execution_authorized = true`, and only when
all required source evidence is fresh and matched.

## Binding Requirements

The lifecycle artifact binds:

```text
schema_version = ntpro.v190_actual_cancel_owner_approval_lifecycle.v1
actual_cancel_safety_contract = docs/rust-cutover/release/v0_19_0_actual_cancel_safety_contract.md
release_manifest = docs/rust-cutover/release/v0_18_1_release_manifest.json
cancel_risk_gate = required
order_lineage_id = expected_order_lineage_id
symbol = expected_symbol
account_label = expected_account_label
venue = required
expected_release_tag = ntpro-rust-only-v0.18.1
manual_approval_id = required for authorizing states
approved_by = required for authorizing states
approval_reason = required for authorizing states
expires_at_unix_ms = required
```

## Fail-Closed Semantics

The lifecycle blocks before any cancel send when any of these are true:

```text
missing CLI confirmation
V190-002 safety contract missing required tokens
release manifest schema/product/tag/boundary mismatch
cancel risk gate not ready
order lineage mismatch
symbol mismatch
account label mismatch
approval state is created/expired/used/rejected/audited
approval expired by timestamp
manual owner identity, approval id, or reason missing
used/rejected/audited state lacks audit evidence
```

`used`, `rejected`, and `audited` states must record audit evidence. A `used`
approval sets:

```text
approval_consumed = true
approval_consumed_before_send = true
approval_consumed_after_send = true
audit_evidence_recorded = true
approval_execution_authorized = false
```

## Boundary

This lifecycle artifact does not implement:

```text
cancel executor
adapter cancel call
network cancel request
Dashboard approve button
Dashboard cancel button
automatic cancel
bulk cancel
retry / replace / amend / flatten
production order submit lifecycle
long-lived permission system
multi-owner quorum
```

## Validation

```text
cargo test -p nautilus-cli owner_approval --lib
cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_owner_approval_lifecycle_options --lib
cargo clippy -p nautilus-cli --all-targets -- -D warnings
scripts/ai/verify_fast.sh
git diff --check
```
