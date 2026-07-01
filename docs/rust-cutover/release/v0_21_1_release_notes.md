# NTPRO Rust-only v0.21.1 Release Notes

Date: 2026-07-01
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.21.1`
Release name: `NTPRO Rust-only v0.21.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.21.1`
Base release: `ntpro-rust-only-v0.21.0`

## Scope

v0.21.1 is the Unified Read Model Foundation Hardening Patch. It hardens the
v0.21.0 Unified Read Model Foundation before v0.22.0 Trader Terminal workbench
work starts.

This patch is not the Trader Terminal workbench. This patch does not add submit capability.

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

## Trader Terminal Read-Model Runtime Bridge

V211-005 adds a local Dashboard runtime bridge for the canonical unified read
model artifact:

```text
canonical artifact = v0_21/unified_read_model_snapshot.json
Dashboard key = read_model_runtime
release target = scripts/ai/verify_release.sh v21.1-trader-terminal-read-model-bridge
```

The bridge reads `contract_version`, `schema_version`, `snapshot_id`,
`snapshot_kind`, `health_status`, `freshness`, `source_provenance`,
`redaction`, `blocking_reasons`, and the required `account`, `positions`,
`orders`, `fills`, `risk`, and `lifecycle_status` component envelopes. It keeps
missing or invalid runtime evidence visible instead of treating the foundation
as healthy:

```text
missing_artifact
schema_mismatch
stale_artifact
component_missing
component_unavailable
```

Plain Chinese summary: V211-005 让 Dashboard 读取本地 canonical unified read
model artifact，并展示 account、position、order、fill、risk、lifecycle 的基础只读状态。
缺 artifact、schema mismatch、stale、component missing 或 component_unavailable
都会显示 degraded、stale 或 fail_closed，不会显示 healthy。边界仍保持：
`dashboard_order_controls_enabled = false`，且不新增下单、审批、撤单、重试、替换、改单、
平仓或产品级实盘终端能力。

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
scripts/ai/verify_release.sh v21-release-gates
scripts/ai/verify_release.sh v21.1-read-model-projection-replay
scripts/ai/verify_release.sh v21.1-read-model-schema-boundary
scripts/ai/verify_release.sh v21.1-trader-terminal-read-model-bridge
scripts/ai/verify_release.sh v21.1-release-gates
scripts/ai/verify_release.sh v21.1-strict-provenance
scripts/ai/verify_v21_1_release_gates.sh
scripts/ai/verify_v21_1_strict_provenance.sh
```

This gate fails if the Rust read-model projection replay fails, if the release
scope does not mark exactly the promoted cases as executable replay, or if any
remaining schema-only read_model case claims executable replay fields.
The schema-boundary gate fails if any read_model fixture violates the JSON
Schema, or if negative mutations for undeclared fields, sensitive fields,
forbidden boundary flags, or invalid source truth claims unexpectedly pass.
The Trader Terminal bridge gate fails if the Dashboard cannot read the
canonical read-model artifact, or if missing, stale, schema-mismatch, component
missing, or component unavailable states stop being represented as non-healthy
read-only runtime statuses.

## Release Gates And Strict Provenance

V211-006 adds the final v0.21.1 release gates:

```text
V211-001
V211-002
V211-003
V211-004
V211-005
V211-006
scripts/ai/verify_release.sh v21.1-release-gates
scripts/ai/verify_release.sh v21.1-strict-provenance
scripts/ai/verify_v21_1_strict_provenance.sh
```

The release gate fails closed when any V211 evidence file is missing. It also
checks the published v0.21.0 closeout manifest, the v0.21.1 release manifest,
the v0.22.0 milestone dependency proof, and the V220 issue body/comment
dependency proof.

## v0.22.0 Dependency

The next capability track is `v0.22.0`. V220 work remains blocked until all
V211 issues close and this v0.21.1 release evidence is published. After that
publication proof exists, v0.22.0 can start from its own scoped issue order.

## Boundary

v0.21.1 explicitly does not include:

- new production submit capability;
- production order mutation;
- implicit retry;
- automatic cancel;
- automatic remediation;
- retry, replace, amend, correction, or flatten;
- strategy-driven production execution;
- multi-account or multi-venue execution expansion;
- Trader Terminal workbench;
- product-grade live trading terminal readiness;
- Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or order-ticket controls.
