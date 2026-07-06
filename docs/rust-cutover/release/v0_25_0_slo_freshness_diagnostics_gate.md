# v0.25.0 SLO freshness diagnostics gate

Date: 2026-07-05
Executor: Codex
GitHub issue: `#784`

## Summary

V250-007 adds a read-only diagnostics gate for the v0.25.0 monitoring,
incident, runbook/audit, and DR preview surface.

Plain Chinese summary: 本变更让 v25 Dashboard 只读面板明确显示 SLO、
freshness threshold、staleness reason、diagnostic severity、source truth、release
provenance 和 no-action boundary。超过阈值、unknown adapter truth、release
provenance drift、partial projection 或 action flag 都不会被渲染成 healthy。

## Boundary

- No submit/cancel/retry/replace/amend/flatten controls.
- No order ticket.
- No adapter send.
- No live exchange request.
- No automated remediation or restoration.
- No product-grade live trading terminal claim.

## Evidence

- Golden trace: `tests/golden/v250_slo_freshness_diagnostics_gate.jsonl`
- Release verifier:
  `scripts/ai/verify_v25_slo_freshness_diagnostics_gate.sh`
- Release stage:
  `scripts/ai/verify_release.sh v25-slo-freshness-diagnostics-gate`
- Dashboard runtime:
  `crates/cli/src/dashboard.rs`

## Rollback

Revert the V250-007 PR to remove the diagnostics gate fields, verifier, golden
trace, release scope entries, and docs. V250-006 remains the prior read-only
Dashboard surface.
