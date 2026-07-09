# v0.28.0 Trader Terminal Backend API Contract Handoff

Date: 2026-07-08
Executor: Codex
Task: `V280-007` / GitHub issue `#900`
Milestone: `v0.28.0`

## Contract

```text
contract_version = ntpro.v280.trader_terminal_backend_api_contract_handoff.v1
schema_version = ntpro.v280.trader_terminal_backend_api_contract_artifact.v1
release_scope = backend_closure_product_operations_runtime_finalization_only
backend_module = trader_terminal_backend_api_contract_handoff
backend_module_status = runtime_closed
depends_on = V280-001,V280-002,V280-003,V280-004,V280-005,V280-006,V220-007,V250-006,V260-007
deterministic_artifact_path = docs/rust-cutover/release/v0_28_0_trader_terminal_backend_api_contract_artifact.json
handoff_mode = deterministic_read_only_backend_api_contract
read_only_api_contract_required = true
response_schema_required = true
redaction_required = true
freshness_required = true
failure_semantics_required = true
breaking_change_migration_note_required = true
order_ticket_enabled = false
submit_controls_enabled = false
operation_controls_enabled = false
trading_controls_enabled = false
product_grade_trading_terminal_claim = false
```

V280-007 closes the Trader Terminal backend API handoff contract by defining
source-controlled read-only API/artifact responses for backend closure status,
provenance drill-down, audit entries, telemetry health, permissions, and
deployment state. Follow-up product/frontend work can consume this contract
without interpreting ad hoc release evidence files.

## Runtime Artifact Path

```text
artifact = docs/rust-cutover/release/v0_28_0_trader_terminal_backend_api_contract_artifact.json
validator = scripts/ai/verify_v28_trader_terminal_backend_api_contract_handoff.sh
release stage = scripts/ai/verify_release.sh v28-trader-terminal-backend-api-contract-handoff
matrix module = trader_terminal_backend_api_contract_handoff
matrix classification = runtime-closed
closure_mode = deterministic_artifact_replay
runtime_closed_label = runtime-closed (artifact replay)
```

## Handoff API Contracts

```text
GET /api/v28/backend-closure/status
GET /api/v28/provenance/drilldown
GET /api/v28/audit/entries
GET /api/v28/telemetry/health
GET /api/v28/permissions/snapshot
GET /api/v28/deployment/state
```

Each contract row must carry:

```text
read_only = true
allowed_methods = GET
mutating_methods_allowed = false
response_schema = required fields and schema id
redaction_status = redacted
freshness_status = fresh | stale
failure_semantics = missing_source, malformed_response, stale_source, forbidden_controls
source_refs = source-controlled release artifacts or evidence
verification_commands = release gate commands
operation_controls_enabled = false
trading_controls_enabled = false
order_ticket_enabled = false
manual_operation_submit_allowed = false
```

## Response Semantics

```text
all contracts ready and fresh => trader_terminal_backend_api_contract_ready
contract degraded with reasons => degraded_partial_response
contract stale with reasons => degraded_stale_response
missing source reference => fail_closed_missing_source
malformed response schema => fail_closed_malformed_response
unredacted payload => fail_closed_unredacted_payload
forbidden controls => fail_closed_forbidden_controls
```

## Required-False Operation Boundary

```text
default_submit_allowed = false
submit_order_allowed = false
cancel_order_allowed = false
retry_order_allowed = false
replace_order_allowed = false
amend_order_allowed = false
flatten_position_allowed = false
order_ticket_enabled = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
network_attempted = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
trader_terminal_submit_controls_enabled = false
manual_operation_submit_allowed = false
automatic_remediation_allowed = false
retry_scheduler_enabled = false
product_grade_trading_terminal_claim = false
```

## Boundary Statement

The Trader Terminal backend API handoff is a read-only contract for future
product/frontend work. It may expose backend closure state, provenance, audit,
telemetry, permissions, and deployment state, but it cannot expose order
tickets, submit/cancel/retry/replace/amend/flatten controls, adapter sends,
live exchange access, automatic remediation, or product-grade trading terminal
readiness.
