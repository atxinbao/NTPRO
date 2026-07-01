# v0.21.0 Unified Read Model Contract

Date: 2026-06-30
Executor: Codex
Task: `V210-001`
GitHub issue: `#652`
Status: CONTRACT BASELINE

## Purpose

This document defines the v0.21.0 unified read model contract for account,
position, order, fill, risk, and lifecycle status projections. It is a
read-only foundation contract. It does not add submit capability, Dashboard
order controls, retry/replace/amend/flatten, or product-grade live trading
terminal claims.

Plain Chinese summary: 本文档只定义统一 read model 的输入输出契约。它要求每个
snapshot 都能说明来源、血缘、时间点和新鲜度；缺 source、lineage、freshness 时必须
fail closed，不能标记 healthy。它不新增下单能力，不新增 Dashboard 操作按钮。

## Contract Version

```text
contract_version = ntpro.v210.unified_read_model.v1
schema = docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json
schema smoke = tests/golden/read_model_contract_schema.jsonl
validator = scripts/ai/verify_v21_read_model_contract.sh
account component = docs/rust-cutover/release/v0_21_0_account_snapshot_read_model.md
position component = docs/rust-cutover/release/v0_21_0_position_read_model.md
order lifecycle component = docs/rust-cutover/release/v0_21_0_order_lifecycle_read_model.md
fill/execution component = docs/rust-cutover/release/v0_21_0_fill_execution_read_model.md
risk projection component = docs/rust-cutover/release/v0_21_0_risk_state_projection.md
Trader Terminal read-only dashboard foundation = docs/rust-cutover/release/v0_21_0_trader_terminal_readonly_dashboard.md
```

## Snapshot Identity

Every unified read model snapshot must include:

```text
snapshot_id = stable deterministic ID for this projected snapshot
contract_version = ntpro.v210.unified_read_model.v1
as_of_unix_ns = newest event/read timestamp represented by the snapshot
created_at_unix_ns = local projection creation timestamp
scope.account_id = redacted/stable account identity or unavailable marker
scope.venue = venue identifier or multi-venue-unavailable marker
scope.instrument_ids = sorted instrument IDs represented by the snapshot
```

Rules:

- `snapshot_id` must be deterministic for a fixture or evidence artifact.
- `as_of_unix_ns` must not be newer than the newest source event/read.
- `created_at_unix_ns` must not be used as exchange truth.
- Raw account IDs may appear only when already redacted or explicitly scoped as
  stable non-secret identifiers.

## Required Top-Level Fields

```text
contract_version
schema_version
snapshot_id
snapshot_identity
as_of_unix_ns
health_status
freshness
source_provenance
lineage
components
blocking_reasons
redaction
capability_boundary
```

Allowed `health_status` values:

```text
healthy
degraded
fail_closed
```

`healthy` is allowed only when every required component has source provenance,
lineage, freshness, and redaction status.

## Components

The minimal v0.21 unified view must be able to represent:

```text
components.account
components.positions
components.orders
components.fills
components.risk
components.lifecycle_status
```

Every component must include:

```text
component_status = healthy | degraded | fail_closed | unavailable
source_provenance
lineage
freshness
redaction
data
diagnostics
```

The component `data` payload may be empty for a contract-only task, but the
component envelope must still exist so later V210 tasks can add data fields
without changing snapshot identity or fail-closed semantics.

## Source Provenance

Every snapshot and component must record:

```text
source_type = fixture | artifact | exchange_readback | adapter_runtime | manual_evidence | unavailable
source_ref = deterministic artifact/event/read reference
captured_at_unix_ns = timestamp of source capture
redaction_state = redacted | no_sensitive_fields | unavailable
exchange_truth = true | false
adapter_runtime_integrated = true | false
```

Rules:

- Missing `source_provenance` prevents `healthy`.
- `exchange_truth=true` requires a source reference that identifies the
  readback or adapter evidence path.
- `adapter_runtime_integrated=false` must be visible when a value is fixture or
  manual evidence only.
- Raw exchange responses, headers, signed URLs, API keys, API secrets, and
  unrestricted payloads must not be stored in the read model.

## Lineage

Every snapshot and component must record:

```text
lineage.input_refs = source refs consumed by the projection
lineage.transform = deterministic transform name/version
lineage.parent_snapshot_ids = prior snapshots used, if any
lineage.lossy_fields = fields intentionally omitted or redacted
```

Rules:

- Missing `lineage` prevents `healthy`.
- `lossy_fields` must name omitted sensitive/raw fields.
- A derived component must not hide ambiguous or conflicting parent evidence.

## Freshness

Every snapshot and component must record:

```text
freshness.status = fresh | stale | missing | ambiguous
freshness.observed_age_ms
freshness.max_age_ms
freshness.as_of_unix_ns
freshness.checked_at_unix_ns
```

Rules:

- Missing `freshness` prevents `healthy`.
- `freshness.status=stale`, `missing`, or `ambiguous` prevents `healthy`.
- Freshness must be computed against the source `as_of_unix_ns`, not against
  local projection creation time alone.

## Fail-Closed Rules

The unified read model must set `health_status=fail_closed` when any of these
are true:

```text
missing_snapshot_source_provenance
missing_snapshot_lineage
missing_snapshot_freshness
missing_component_source_provenance
missing_component_lineage
missing_component_freshness
stale_component_freshness
mismatched_snapshot_identity
ambiguous_component_lineage
unredacted_sensitive_field
```

A fail-closed snapshot must include:

```text
blocking_reasons = non-empty array
component_status = fail_closed for every blocking component
capability_boundary.production_order_submission_allowed = false
capability_boundary.production_order_mutation_allowed = false
capability_boundary.dashboard_order_controls_enabled = false
```

## Capability Boundary

The contract must always preserve:

```text
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
dashboard_order_controls_enabled = false
dashboard_approval_controls_enabled = false
dashboard_cancel_controls_enabled = false
dashboard_retry_controls_enabled = false
retry_replace_amend_flatten_allowed = false
product_grade_trading_terminal_claim = false
```

## Trader Terminal Dashboard Foundation

The v0.21 Trader Terminal dashboard foundation is a read-only projection over
the unified account, position, order, fill, risk, and lifecycle components. It
may display account, position, order, fill, risk, and audit/provenance
diagnostics, but it must preserve:

```text
foundation_only = true
read_only = true
no_submit_controls = true
view transform = ntpro.v210.trader_terminal_readonly_dashboard.v1
```

It must not expose submit, approval, cancel, retry, replace, amend, flatten,
order ticket, automatic repair, automatic execution, or product-grade trading
terminal behavior.

## Validation

Use:

```bash
scripts/ai/verify_release.sh v21-read-model-contract
```

The validator checks the JSON schema, validates the schema-only golden trace
envelope, and asserts that missing lineage/source/freshness snapshots cannot be
marked healthy.
