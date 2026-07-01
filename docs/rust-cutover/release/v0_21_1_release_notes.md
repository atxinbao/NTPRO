# v0.21.1 Release Notes

Date: 2026-07-01
Executor: Codex
Status: PATCH RELEASE NOTES DRAFT

## Scope

v0.21.1 hardens the v0.21.0 Unified Read Model Foundation before v0.22.0
Trader Terminal workbench work starts.

Plain Chinese summary: v0.21.1 是 read model 基础的补丁版本。它先修正
`health_status` 语义，再把关键 read_model golden traces 提升为 Rust 可执行 replay。
它还收紧 JSON Schema 边界，防止未声明字段、敏感字段或越权 boundary flags
静默穿透。它不新增下单、撤单、重试、替换、改单、平仓、自动修复或产品级实盘终端能力。

## Executable Read-Model Replay

V211-003 promotes these read_model cases from `schema_only_scoped` to
`executable_replay`:

```text
read_model.account_snapshot.fresh.001
read_model.account_snapshot.stale.001
read_model.order_lifecycle.matched.001
read_model.order_lifecycle.missing_ledger.001
read_model.risk_state.healthy.001
read_model.risk_state.mismatch.001
read_model.dashboard.readonly_complete.001
read_model.dashboard.missing_evidence_degraded.001
```

Replay harness:

```text
cargo test -p nautilus-cli --test golden_trace_read_model_projection
```

The Rust replay derives the expected output event from the input read-model
snapshot and compares it to the golden expected event. The release scope now
records 8 executable read_model replay cases and 24 remaining schema-only
read_model cases.

## JSON Schema Boundary

V211-004 tightens the unified read-model JSON Schema:

```text
source_provenance.additionalProperties = false
redaction.additionalProperties = false
capability_boundary.additionalProperties = false
component.additionalProperties = false
component.data.additionalProperties = false
dashboard submit/replace/amend/flatten flags = explicit false
trader_terminal_order_ticket_enabled = false
trader_terminal_live_trading_claim = false
```

`exchange_truth=true` and `adapter_runtime_integrated=true` now require
exchange/readback or adapter/runtime source provenance. Fixture, manual, and
unavailable sources cannot claim exchange truth or adapter runtime integration.
The schema gate also rejects partial component snapshots that try to advertise
top-level unified `healthy`.

## Remaining Schema-Only Cases

The remaining schema-only read_model cases are intentionally not promoted yet.
They stay explicitly marked as remaining schema-only follow-up for V211-004
through V211-006, including:

```text
read_model.contract.*
read_model.position.*
read_model.order_lifecycle.unknown_response.001
read_model.order_lifecycle.readback_mismatch.001
read_model.order_lifecycle.duplicate_attempt.001
read_model.fill_execution.*
read_model.risk_state.risk_visible.001
read_model.risk_state.manual_review.001
read_model.risk_state.halted.001
read_model.risk_state.stale.001
read_model.dashboard.forbidden_controls_blocked.001
```

## Validation

Use:

```bash
scripts/ai/verify_release.sh v21.1-read-model-projection-replay
scripts/ai/verify_release.sh v21.1-read-model-schema-boundary
```

This gate fails if the Rust read-model projection replay fails, if the release
scope does not mark exactly the promoted cases as executable replay, or if any
remaining schema-only read_model case claims executable replay fields.
The schema-boundary gate fails if any read_model fixture violates the JSON
Schema, or if negative mutations for undeclared fields, sensitive fields,
forbidden boundary flags, or invalid source truth claims unexpectedly pass.
