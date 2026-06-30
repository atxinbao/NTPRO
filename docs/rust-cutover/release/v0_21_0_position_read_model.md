# v0.21.0 Position Read Model

Date: 2026-06-30
Executor: Codex
Task: `V210-003`
GitHub issue: `#654`
Status: COMPONENT CONTRACT

## Purpose

This document defines the v0.21 position read-model component. It extends the
unified read model with a redacted, auditable, read-only position view for
position count, direction, quantity, cost basis, mark price, notional, and risk
projection inputs.

Plain Chinese summary: 本任务只定义持仓 read model。它可以展示 long、short、flat
持仓状态和风险输入，但不做自动平仓、自动修复、自动调仓，也不新增 Dashboard 操作控件
或生产交易权限。

## Contract Surface

```text
component = components.positions
contract_version = ntpro.v210.unified_read_model.v1
component transform = ntpro.v210.position_read_model.v1
validator = scripts/ai/verify_v21_position_read_model.sh
release target = scripts/ai/verify_release.sh v21-position-read-model
golden trace = tests/golden/read_model_position_schema.jsonl
```

## Position Identity

Every position row must record:

```text
instrument_identity.instrument_id
instrument_identity.venue
instrument_identity.symbol
instrument_identity.quote_currency
position_id = deterministic redacted position identifier
account_id_ref = redacted account identity reference
```

Rules:

- Instrument identity must be stable and auditable.
- `account_id_ref` must match the snapshot account identity.
- A mismatch between account lineage and position lineage must fail closed.

## Precision

Position quantity fields must record:

```text
quantity
quantity_precision
instrument_quantity_precision
precision_status = valid | mismatch
```

Rules:

- A precision mismatch prevents `healthy`.
- Quantity and notional are string decimals; no binary floating point values are
  allowed in the read-model fixture.
- A flat position uses `net_position_side=flat`, `position_count=0`, and
  `quantity=0`.

## Risk Projection Inputs

The position component may expose only read-only risk inputs:

```text
current_position_notional
projected_position_notional
max_position_notional
risk_state
blocking_reasons
```

It must not perform or enable:

```text
auto_flatten_position = not included
automatic_position_repair = not included
order_submission = not included
order_mutation = not included
dashboard_position_controls = not included
```

## Fail-Closed Rules

The position component must be `fail_closed` when any of these are true:

```text
position_quantity_precision_mismatch
stale_position_source
missing_position_source_provenance
account_position_lineage_mismatch
ambiguous_position_side
unredacted_position_payload
```

Fail-closed position snapshots must keep:

```text
health_status = fail_closed
components.positions.component_status = fail_closed
capability_boundary.production_order_submission_allowed = false
capability_boundary.production_order_mutation_allowed = false
capability_boundary.dashboard_order_controls_enabled = false
capability_boundary.auto_flatten_position_allowed = false
capability_boundary.automatic_position_repair_allowed = false
```

## Validation

Use:

```bash
scripts/ai/verify_release.sh v21-position-read-model
```
