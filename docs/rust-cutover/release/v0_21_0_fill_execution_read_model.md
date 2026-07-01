# v0.21.0 Fill And Execution Read Model

Date: 2026-07-01
Executor: Codex
Task: `V210-005`
GitHub issue: `#656`
Status: COMPONENT CONTRACT

## Purpose

This document defines the v0.21 fill/execution read-model component. It
projects fill and execution evidence into the unified read model as a read-only
view for fill identity, execution identity, order linkage, quantity/price
precision, source provenance, duplicate detection, partial-fill state, and
reconciliation status.

Plain Chinese summary: 本任务只定义成交和执行回报的只读 read model。它能展示 fill
id、execution id、order linkage、数量/价格精度、来源、去重和 reconciliation 状态，
但不新增执行算法、不自动补单、不自动撤单、不自动修复，也不新增 Dashboard 操作控件。

## Contract Surface

```text
component = components.fills
contract_version = ntpro.v210.unified_read_model.v1
component transform = ntpro.v210.fill_execution_read_model.v1
validator = scripts/ai/verify_v21_fill_execution_read_model.sh
release target = scripts/ai/verify_release.sh v21-fill-execution-read-model
golden trace = tests/golden/read_model_fill_execution_schema.jsonl
```

## Fill And Execution Identity

Every fill/execution row must record:

```text
fill_id
execution_id
order_id
client_order_id
order_linkage_status
source_provenance_ref
```

Rules:

- `fill_id` must be stable and deterministic for the read-model source.
- `execution_id` must link to exactly one execution evidence record.
- `order_id` and `client_order_id` must link back to the order lifecycle read
  model when available.
- Missing or ambiguous order linkage prevents `healthy`.
- Fill and execution identifiers are redacted references, not raw exchange
  payloads.

## Precision And Quantity

The component must expose quantity and price as string decimals:

```text
quantity
cumulative_quantity
remaining_quantity
quantity_precision
price
price_precision
precision_status = valid | mismatch
```

Rules:

- Binary floating point values are not accepted in the read-model fixture.
- Partial fills must keep `remaining_quantity` visible for downstream risk
  state projection.
- Quantity or price precision mismatches prevent `healthy`.

## Reconciliation

The fill/execution component must derive a read-only reconciliation state:

```text
reconciliation_status = reconciled | partial_fill_visible | duplicate_rejected | missing_order_linkage | stale_execution_source | ambiguous_source
risk_projection_input.fill_reconciliation_status
risk_projection_input.realized_fill_quantity
risk_projection_input.remaining_order_quantity
risk_projection_input.risk_state
```

Rules:

- Reconciled fills may feed downstream risk projection as read-only input.
- Partial fills may be displayed as `degraded` but must not trigger automatic
  execution, retry, cancel, repair, or flatten behavior.
- Duplicate fills fail closed and must not be counted twice.
- Missing order linkage, stale execution source, or ambiguous source provenance
  fails closed.

## Fail-Closed Rules

The fills component must be `fail_closed` when any of these are true:

```text
duplicate_fill
missing_order_linkage
stale_execution_source
ambiguous_fill_source
fill_quantity_precision_mismatch
unredacted_fill_payload
```

Fail-closed fill snapshots must keep:

```text
health_status = fail_closed
components.fills.component_status = fail_closed
capability_boundary.production_order_submission_allowed = false
capability_boundary.production_order_mutation_allowed = false
capability_boundary.dashboard_order_controls_enabled = false
capability_boundary.dashboard_fill_controls_enabled = false
capability_boundary.execution_algorithm_allowed = false
capability_boundary.automatic_fill_repair_allowed = false
capability_boundary.automatic_reconciliation_repair_allowed = false
```

## Dashboard Boundary

Dashboard may display fill identity, execution identity, order linkage status,
duplicate/partial-fill flags, reconciliation status, and risk projection input.
Dashboard must not expose submit, cancel, retry, replace, amend, flatten,
execution algorithm, fill repair, reconciliation repair, or approval controls
for this read-model scope.

## Validation

Use:

```bash
scripts/ai/verify_release.sh v21-fill-execution-read-model
```
