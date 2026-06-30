# v0.21.0 Account Snapshot Read Model

Date: 2026-06-30
Executor: Codex
Task: `V210-002`
GitHub issue: `#653`
Status: COMPONENT CONTRACT

## Purpose

This document defines the v0.21 account snapshot read-model component. It
extends the unified read model baseline with account-level balances,
available-funds, margin, and risk summary fields as read-only evidence.

Plain Chinese summary: 本任务只把账户快照整理成只读 read model。它可以展示账户状态、
余额条目数量、可用资金和保证金摘要，但不能保存凭证、原始交易所 payload，也不能新增
资金划转、账户配置修改或下单权限控制。

## Contract Surface

The account snapshot component lives at:

```text
components.account
contract_version = ntpro.v210.unified_read_model.v1
component transform = ntpro.v210.account_snapshot_read_model.v1
validator = scripts/ai/verify_v21_account_snapshot_read_model.sh
release target = scripts/ai/verify_release.sh v21-account-snapshot-read-model
golden trace = tests/golden/read_model_account_snapshot_schema.jsonl
```

## Source And Provenance

Every account snapshot read-model component must record:

```text
source_provenance.source_type = artifact | exchange_readback | fixture
source_provenance.source_ref = deterministic artifact or read reference
source_provenance.captured_at_unix_ns = source capture timestamp
source_provenance.redaction_state = redacted
source_provenance.exchange_truth = true | false
source_provenance.adapter_runtime_integrated = true | false
lineage.input_refs includes the account snapshot source ref
lineage.lossy_fields names omitted raw balances, permissions, UID, headers, signed query, signed URL, and raw response fields
freshness.status = fresh | stale | missing | ambiguous
```

Rules:

- Missing account `source_provenance` prevents `healthy`.
- `freshness.status=stale`, `missing`, or `ambiguous` prevents `healthy`.
- The read model may record counts and redacted summary values only.
- `exchange_truth=true` is allowed only when the source reference names a
  redacted readback or artifact path.

## Account Data

Fresh account snapshots may expose only the redacted summary fields below:

```text
account_status
balance_entry_count
available_balance
margin_available
equity
risk_state
response_shape
response_shape_validated
```

No raw asset rows, account permissions, UID, headers, API key, API secret,
signature, signed query, signed URL, or unrestricted raw exchange response may
appear in the read model.

## Fail-Closed Rules

The account snapshot component must be `fail_closed` when any of these are
true:

```text
missing_account_source_provenance
stale_account_freshness
ambiguous_account_freshness
missing_account_freshness
unredacted_sensitive_field
raw_account_payload_persisted
```

A fail-closed account snapshot must keep:

```text
health_status = fail_closed
components.account.component_status = fail_closed
blocking_reasons = non-empty
capability_boundary.production_order_submission_allowed = false
capability_boundary.production_order_mutation_allowed = false
capability_boundary.dashboard_order_controls_enabled = false
```

## Dashboard Boundary

The Dashboard may display account state, status, artifact path, endpoint class,
shape status, and boundary flags. It must not provide account operation
controls:

```text
funds_transfer = not included
account_configuration_mutation = not included
order_permission_control = not included
dashboard_order_controls_enabled = false
dashboard_approval_controls_enabled = false
dashboard_cancel_controls_enabled = false
dashboard_retry_controls_enabled = false
```

Existing read-only Dashboard evidence is in `crates/cli/src/dashboard.rs`:
`account_snapshot_status`, `account_snapshot_endpoint_class`, and
`account_snapshot_path`.

## Validation

Use:

```bash
scripts/ai/verify_release.sh v21-account-snapshot-read-model
```
