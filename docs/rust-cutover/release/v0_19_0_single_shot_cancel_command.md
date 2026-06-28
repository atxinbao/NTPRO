# v0.19.0 Single-Shot Actual Cancel Command

Date: 2026-06-27
Executor: Codex
Milestone: `v0.19.0`
Task: `V190-004`
Status: REVIEW_REQUIRED IMPLEMENTATION CONTRACT

## Summary

This document defines the v0.19.0 owner-approved single-shot actual cancel
command. The command stays offline by default and only records a real cancel
attempt when the owner explicitly requests `--manual-online`, every CLI
confirmation is present, all upstream evidence matches, and the required env
gates and credentials are available.

Plain Chinese summary: 这次实现真正撤单命令的唯一入口，但默认仍然不发请求。大白话：
只有 owner 手动提供 raw order id、打开 `--manual-online`、同时提供 owner approval、risk gate、
release provenance、adapter boundary/capability、CLI 确认和环境变量闸门时，命令才会构造并发送一次
`DELETE /api/v3/order`。它不支持 Dashboard 按钮、不支持自动撤单、不支持 bulk/cancel-all、不支持 retry。

## CLI Surface

```text
nautilus live production-mutation-actual-cancel-single-shot \
  --run-id v190-actual-cancel-single-shot \
  --actual-cancel-safety-contract docs/rust-cutover/release/v0_19_0_actual_cancel_safety_contract.md \
  --release-manifest docs/rust-cutover/release/v0_18_1_release_manifest.json \
  --cancel-risk-gate runs/v180/cancel-risk-gate.json \
  --owner-approval-lifecycle runs/v190/actual-cancel-owner-approval-lifecycle.json \
  --adapter-boundary runs/v190/actual-cancel-executor-adapter-boundary.json \
  --adapter-capability runs/v190/adapter-capability.json \
  --expected-order-lineage-id lineage-v160-single-shot \
  --expected-symbol BTCUSDT \
  --expected-account-label prod-account-redacted \
  --venue binance_spot \
  --order-id-type exchange_order_id \
  --expected-release-tag ntpro-rust-only-v0.18.1 \
  --cancel-order-id OWNER_SUPPLIED_EXCHANGE_ORDER_ID \
  --api-key-env BINANCE_PRODUCTION_LIVE_ALPHA_API_KEY \
  --api-secret-env BINANCE_PRODUCTION_LIVE_ALPHA_API_SECRET \
  --timestamp-ms 1718400000000 \
  --recv-window-ms 5000 \
  --output runs/v190/actual-cancel-single-shot.json \
  --manual-online \
  --allow-production-mutation-actual-cancel-single-shot \
  --confirm-owner-approval \
  --confirm-risk-gate \
  --confirm-release-provenance \
  --confirm-adapter-boundary \
  --confirm-single-shot \
  --confirm-consume-approval-before-send \
  --confirm-readback-required \
  --confirm-no-bulk-cancel \
  --confirm-no-retry \
  --confirm-no-automatic-cancel \
  --confirm-no-dashboard-execution \
  --confirm-no-secret-persistence
```

`--order-id-type client_order_id` uses `--cancel-orig-client-order-id` instead
of `--cancel-order-id`.

## Manual Online Gates

The command reads API credentials only after these env gates are all set to
`1`:

```text
PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_ALLOW
PRODUCTION_MUTATION_SIGNING_MATERIAL_ENV_OWNER_APPROVED
PRODUCTION_MUTATION_HTTP_SEND_ENV_ALLOW
PRODUCTION_MUTATION_HTTP_SEND_ENV_OWNER_APPROVED
PRODUCTION_MUTATION_HTTP_SEND_ENV_SINGLE_SHOT
```

Without `--manual-online`, the command may produce a ready offline artifact but
must not build a live HTTP request or send a cancel.

## Artifact Contract

```text
schema_version = ntpro.v190_actual_cancel_single_shot.v1
artifact_type = actual_cancel_single_shot
status =
  ready_actual_cancel_command_offline_no_send
  actual_cancel_attempt_recorded
  blocked_missing_gate
  blocked_missing_manual_online_gate
  blocked_safety_contract
  blocked_release_provenance
  blocked_source_artifact
  blocked_adapter_capability
request_method = DELETE
request_target = /api/v3/order
request_contract = single_order_cancel_request_v1
single_shot_cancel_allowed = true only when all evidence and manual online gates match
cancel_requests_sent = 0 or 1
production_order_mutations_attempted = 0 or 1
readback_required = true after any attempted send
```

Before any attempted send, the command atomically marks the source owner
approval lifecycle as `used` and clears `approval_execution_authorized`; after
the attempt it records the venue metadata and readback requirement on that same
lifecycle artifact. Reusing the same owner approval path then fail-closes with
`owner_approval_reused` before any executor call.

The cancel attempt artifact records only redacted order identifiers and
metadata. It must not
persist raw order ids, API key values, API secret values, API key headers,
signatures, signed queries, signed URLs, raw request bodies, raw response
bodies, or response headers.

## Required Evidence

The command consumes and validates:

```text
V190-002 actual cancel safety contract
v0.18.1 release manifest
v0.18 cancel risk gate
V190-003 owner approval lifecycle
V190-005 cancel executor adapter boundary
adapter capability declaration
owner-supplied raw order id or original client order id
```

The owner-supplied raw identifier is used only in memory for request signing.
It must match the redacted/hash identifier in the owner approval lifecycle.

## Fail-Closed Semantics

The command blocks before any send when any of these are true:

```text
missing CLI confirmation
missing manual online env gate or API credential env when --manual-online is requested
V190-002 safety contract missing required tokens
release manifest schema/product/tag/provenance mismatch
risk gate not ready or not matched to order/symbol/account
owner approval missing, expired, reused, rejected, audited, or mismatched
adapter boundary not ready or not authorized
adapter capability missing or unsupported
venue or order-id type mismatch
owner-supplied raw order id mismatch
bulk/cancel-all/retry/automatic/Dashboard path requested
```

After any attempted send, the artifact records:

```text
approval_consumed_before_send = true
approval_consumed_after_send = true
approval_state_after_attempt = used
source owner approval lifecycle approval_state = used
source owner approval lifecycle approval_execution_authorized = false
source owner approval lifecycle lifecycle_issues includes owner_approval_reused
readback_required = true
retry_attempted = false
bulk_cancel_allowed = false
dashboard_execution_allowed = false
```

## Boundary

This command does not implement:

```text
Dashboard cancel button
Dashboard owner approval button
automatic cancel
bulk cancel
cancel-all
retry / replace / amend / flatten / remediation
multi-account cancel
multi-strategy cancel
multi-venue cancel
production order submit lifecycle
credential persistence
raw response persistence
```

## Validation

```text
cargo test -p nautilus-cli actual_cancel --lib
cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_single_shot_options --lib
cargo clippy -p nautilus-cli --all-targets -- -D warnings
scripts/ai/verify_fast.sh
scripts/ai/verify_release.sh v19-release-gates once available
git diff --check
```
