# NTPRO Rust-only v0.29.0

Status: RELEASE GATE READY
Tag: `ntpro-rust-only-v0.29.0`
Release name: `NTPRO Rust-only v0.29.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.29.0`
Base release: `ntpro-rust-only-v0.28.1`

v0.29.0 publishes the Backend Production Readiness Foundation track. It closes
the V290 readiness line by proving the v0.28.1 dependency, backend readiness
boundary contract, persistent audit storage, telemetry/SLO ingestion,
permission source, read-only backend API, deployment config/runbook,
monitoring/alert/incident readiness, canary/rollback/DR preflight readiness,
backend production readiness fail-closed hardening, release gates, strict
provenance, and the v0.30.0 backend production go-live candidate handoff.

## Release Scope

```text
V290-000 v0.29.0 intake gate and v0.28.1 dependency proof
V290-001 backend production readiness boundary contract
V290-002 persistent audit storage production readiness
V290-003 telemetry SLO ingestion production readiness
V290-004 permission source production readiness
V290-005 read-only backend API production readiness
V290-006 deployment config and runbook production readiness
V290-007 monitoring alert incident production readiness
V290-008 canary rollback DR preflight readiness
V290-009 backend production readiness fail-closed hardening
V290-010 v29 release gates and v30 go-live candidate handoff
V290-011 v29 hosted release gate JSON payload fix
V290 final release scope issue count = 12
V290 final release scope evidence count = 12
V290 exact milestone issue set = #926-#936, #961
V290 registered corrective-scope exception count = 1
```

## Release Gates

```text
v29 release gates = required
v29 strict provenance = required
backend production readiness boundary contract = required
backend production readiness fail-closed hardening = required
release surface current guard = required
release publication guard = required
release publish after gate = required
hosted release gate success before public GitHub Release = required
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
scripts/ai/verify_release.sh v29-release-gates
scripts/ai/verify_release.sh v29-strict-provenance
scripts/ai/verify_v29_release_gates.sh
scripts/ai/verify_v29_strict_provenance.sh
scripts/ai/check_github_release_published.sh
scripts/ai/publish_ntpro_release_after_gate.sh
```

## Boundary

```text
backend production readiness foundation = complete
backend production go-live = false
production execution runtime = false
product-grade live trading terminal = false
v0.30.0 backend production go-live candidate = next track
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
cancel_order_allowed = false
replace_order_allowed = false
amend_order_allowed = false
flatten_position_allowed = false
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
network_attempted = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
automatic_operation_action_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
backend_go_live_claim = false
product_grade_trading_terminal_claim = false
```

## Next Tracks

The next patch track is `v0.29.1`.
The next capability track is `v0.30.0`.

`v0.30.0` is a backend production go-live candidate track only after v0.29.0
publication evidence exists. It does not inherit default submit, production
order mutation, adapter send, live exchange request, retry scheduler, automatic
remediation, Dashboard/Admin/Trader Terminal trading controls, backend go-live,
or product-grade live trading terminal claims from v0.29.0.
