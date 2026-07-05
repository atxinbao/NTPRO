# v0.24.1 Dashboard Fixture Ref Integrity

Date: 2026-07-05
Executor: Codex
Task: V241-007
GitHub issue: #776

## Summary

V241-007 hardens Dashboard fixture provenance references before the v0.24.1
release gate. The stale `policy_ref` path now points to the real v24 order
intent / execution policy contract, and the gate rejects missing paths or
unresolved anchors.

```text
Task: `V241-007` / GitHub issue `#776`
tests/golden/v240_dashboard_workbench_order_control_preview.json
tests/golden/v241_dashboard_order_control_artifact_ingestion.json
scripts/ai/verify_release.sh v24.1-dashboard-fixture-ref-integrity
policy_ref = docs/rust-cutover/release/v0_24_0_order_intent_execution_policy.md
bad_path_selftest = fail_closed
bad_jsonl_anchor_selftest = fail_closed
bad_markdown_anchor_selftest = fail_closed
dashboard_operation_controls_enabled = false
```

## Resolver Rules

- JSON fixture syntax must be valid.
- Every non-empty `*_ref` must point to an existing source-tree path.
- JSONL anchors resolve through an explicit semantic alias map to a concrete
  `case_id`.
- Markdown anchors resolve to a heading slug or explicit `<a id="..."></a>`.
- Missing provenance is allowed only for the explicit degraded-unavailable
  missing-provenance case.

## Negative Self-Tests

- missing document path fails closed.
- missing JSONL anchor fails closed.
- missing Markdown anchor fails closed.

## Boundary

This gate is evidence integrity only. It does not add Dashboard operation
controls, live control APIs, adapter sends, production mutation, retry
scheduler, or product-grade live terminal status.
