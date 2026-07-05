# v0.24.1 Dashboard Artifact Ingestion Hardening

Date: 2026-07-05
Executor: Codex
Task: V241-005
GitHub issue: #774

## Summary

V241-005 closes the gap between renderer-only Dashboard smoke coverage and the
real artifact ingestion path. The Dashboard now diagnoses v24 order-control
preview artifacts before rendering them, and the v24.1 gate renders the actual
Dashboard JS from raw artifact-derived runtime values.

```text
Task: `V241-005` / GitHub issue `#774`
tests/golden/v241_dashboard_order_control_artifact_ingestion.json
scripts/ai/verify_release.sh v24.1-dashboard-artifact-ingestion
forbidden_true_controls = fail_closed
stale_artifact = not_ready
missing_provenance = not_ready
scope_mismatch = fail_closed
missing_redaction = fail_closed
dashboard_operation_controls_enabled = false
```

## Ingestion Rules

- v24 preview component freshness must be fresh; stale artifacts produce
  `stale_artifact`.
- v24 preview component source provenance must be present.
- v24 preview component redaction and data redaction must be `redacted`.
- ready previews with missing provenance fail closed.
- v24 scope must match the artifact account and venue.
- forbidden operation/control boundary booleans must be explicitly `false`.

## Renderer Boundary

The smoke gate renders `renderTraderTerminalWorkbench` and
`renderReadModelRuntime` from `crates/cli/src/dashboard.rs`. It fails if the
rendered HTML or renderer bodies contain button, form, input, fetch, API order
route, Dashboard action, Workbench action, submit, cancel, retry, replace, amend,
or flatten action surfaces.

## Non-Claims

This is Dashboard artifact ingestion hardening only. It is not a Rust
order-control runtime, not adapter integration, not an execution scheduler, and
not a live trading terminal.

## Gate

`scripts/ai/verify_release.sh v24.1-dashboard-artifact-ingestion` fails closed
if any negative artifact renders as ready, if a malicious boundary true value is
accepted, if stale or missing provenance is ready, if scope or redaction drift is
accepted, or if operation controls appear in the Dashboard renderer.
