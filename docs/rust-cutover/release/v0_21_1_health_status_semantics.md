# v0.21.1 Read Model Health Status Semantics

Date: 2026-07-01
Executor: Codex
Task: `V211-002`
GitHub issue: `#678`
Status: PATCH CONTRACT

## Purpose

This document records the v0.21.1 hardening rule for read-model
`health_status`. It separates local component health from unified snapshot
health and dashboard display state.

Plain Chinese summary: 本补丁只修正 read model 健康状态语义。单个组件可以是
healthy，但这不代表统一 snapshot 已经 healthy；只有 account、positions、orders、
fills、risk、lifecycle_status 全部有来源、血缘、新鲜度、脱敏并且组件状态 healthy
时，顶层 unified snapshot 才能 healthy。Dashboard 可以展示局部可见状态，但必须把
missing / unavailable / degraded 明确展示出来。

## Semantic Boundary

```text
component_snapshot = local component evidence
unified_snapshot = complete required component set
dashboard_view = read-only display view over available evidence
```

Rules:

- `component_snapshot` may set a local `component_status=healthy`; its top-level
  `health_status` must stay `degraded` when required unified components are
  missing or unavailable.
- `unified_snapshot.health_status=healthy` requires all required components:
  `account`, `positions`, `orders`, `fills`, `risk`, and `lifecycle_status`.
- Each required unified component must have `source_provenance`, `lineage`,
  `freshness.status=fresh`, `redaction`, and `component_status=healthy`.
- `dashboard_view` may display partial evidence, but missing or unavailable
  data must remain visible as degraded/missing/unavailable and must not be
  promoted to healthy.
- Missing provenance, lineage, freshness, or redaction on a required unified
  component is fail-closed evidence.

## Corrected Fixtures

The following local component fixtures now preserve local healthy component
state while marking the top-level snapshot as degraded:

```text
read_model.account_snapshot.fresh.001
read_model.position.long.001
read_model.position.short.001
read_model.position.flat.001
read_model.order_lifecycle.matched.001
read_model.fill_execution.reconciled.001
read_model.risk_state.healthy.001
```

Full unified healthy remains covered by
`read_model.contract.healthy_minimal.001` and by the dedicated V211 semantics
fixture `read_model.health_status.unified_snapshot.full_healthy.001`.

The dedicated V211 trace lives under
`tests/golden/v211/read_model_health_status_semantics_schema.jsonl` so the
published v0.21.0 flat golden-trace release scope remains unchanged.

## Validation

Use:

```bash
scripts/ai/verify_release.sh v21.1-health-status-semantics
```

The validator checks:

- partial component snapshot;
- full unified healthy;
- missing component dashboard degraded;
- fail-closed missing evidence;
- existing v21 component, unified, and dashboard fixtures.
