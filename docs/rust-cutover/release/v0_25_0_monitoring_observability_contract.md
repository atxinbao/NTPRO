# v0.25.0 Monitoring Observability Contract

Date: 2026-07-05
Executor: Codex
Task: `V250-001`
GitHub issue: `#778`
Milestone: `v0.25.0`
Status: LOCAL VALIDATION PASSED

## Summary

V250-001 defines the read-only monitoring evidence contract for v0.25.0. The
contract separates monitoring truth from exchange truth and adapter runtime
truth, then makes `healthy` display eligibility depend on complete provenance,
freshness, redaction, component health, and no-side-effect boundaries.

Plain Chinese summary: v0.25.0 的 monitoring / observability 只读合同规定：
监控证据只能说明运行观测状态，不能宣称 exchange truth 或 adapter runtime truth，也不能
产生任何交易操作副作用。缺 provenance、缺 freshness、stale、partial、缺 redaction
都不能显示 healthy；redaction breach、truth 边界越界、adapter send/live exchange
request 或任何 operation side effect 都必须 fail closed。

## Contract Fields

```text
contract_version = ntpro.v250.monitoring_observability.v1
schema_version = ntpro.v250.monitoring_observability.schema.v1
runtime_health_status = healthy | degraded | fail_closed
component_health = healthy | partial | degraded | unavailable | fail_closed
freshness.status = fresh | stale | missing
source_provenance = required for runtime and every component
redaction_state = redacted required; missing degrades, breach fails closed
monitoring_truth_scope = runtime_monitoring_evidence_only
exchange_truth_claim = false
adapter_runtime_truth_claim = false
operation_side_effects = all false
```

## Status Rules

```text
healthy =
  source provenance complete
  and freshness complete/fresh
  and redaction complete/redacted
  and every component is healthy
  and monitoring truth only
  and exchange truth claim is false
  and adapter runtime truth claim is false
  and no operation side effect is allowed

degraded_missing_provenance =
  any runtime/component source provenance missing

degraded_stale_or_partial =
  runtime or component freshness stale/missing
  or any component health partial/degraded/unavailable

fail_closed_redaction_breach =
  runtime or component redaction breach/raw/unredacted

fail_closed_boundary_violation =
  exchange truth claim, adapter runtime truth claim, live exchange request,
  adapter send, retry scheduler, automatic remediation, or operation side
  effect is present
```

## Monitoring Truth Boundary

Monitoring artifacts are read-only runtime evidence. They may summarize account,
orders, fills, risk, and order-control preview telemetry, but they do not prove
exchange state, adapter-integrated runtime state, order execution authorization,
or remediation eligibility.

```text
monitoring_truth = allowed read-only evidence
exchange_truth = prohibited claim in V250-001
adapter_runtime_truth = prohibited claim in V250-001
trading_authorization = prohibited claim in V250-001
operation_side_effect = prohibited in V250-001
```

## Golden Trace Coverage

```text
read_model.monitoring_observability.healthy_runtime_truth.001 = healthy allowed only with complete evidence
read_model.monitoring_observability.missing_source_provenance_degraded.001 = degraded, not healthy
read_model.monitoring_observability.stale_partial_degraded.001 = degraded, not healthy
read_model.monitoring_observability.redaction_breach_fail_closed.001 = fail closed
read_model.monitoring_observability.side_effect_boundary_fail_closed.001 = fail closed
```

## Evidence

Validation is recorded in `docs/rust-cutover/evidence/V250-001.md` and
`verification.md`. The release replay manifest records these cases as
`validator_executable_replay`; this is a monitoring contract verifier, not a
runtime adapter integration claim.

## Intake Dependency Proof

V250-001 also aligns `scripts/ai/verify_v25_intake_gate.sh` with the established
V24 intake model. After #777 merges, `origin/main` legitimately advances beyond
the v0.24.1 release tag, so the dependency check requires
`ntpro-rust-only-v0.24.1` to be an ancestor of current `origin/main` rather than
an exact match.
