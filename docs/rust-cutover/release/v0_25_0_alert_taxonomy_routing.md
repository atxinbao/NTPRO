# v0.25.0 Alert Taxonomy And Routing Evidence

Date: 2026-07-05
Executor: Codex
Task: `V250-002`
GitHub issue: `#779`
Milestone: `v0.25.0`
Status: LOCAL VALIDATION PASSED

## Summary

V250-002 defines the v0.25.0 alert taxonomy and routing evidence contract.
Alerts are monitoring evidence only: they classify severity, category, source,
scope, dedupe key, freshness, acknowledgement requirement, provenance, and a
manual/evidence routing target without triggering any real operation.

Plain Chinese summary: 本任务只定义告警证据和只读 routing。告警可以覆盖 stale data、
missing provenance、risk fail-closed、order-control preview blocked、release/gate
drift 等类型，并说明应该进入哪个人工处理入口；但它不接入外部 paging，不执行自动
remediation，不改变 order lifecycle，也不允许任何 submit/cancel/retry/replace/
amend/flatten 或 live exchange/adapter send。

## Contract Fields

```text
contract_version = ntpro.v250.alert_taxonomy_routing.v1
schema_version = ntpro.v250.alert_taxonomy_routing.schema.v1
severity = info | warning | critical | halt
category = stale_data | missing_provenance | risk_fail_closed | order_control_preview_blocked | release_gate_drift
source = monitoring_observability | read_model_runtime | risk_projection | order_control_preview | release_gate
scope = account_key / strategy_key / venue_node_key / isolation_scope_key
dedupe_key = required stable key
freshness.status = fresh required for the alert artifact itself
ack_required = boolean
source_provenance = required
redaction_state = redacted required
routing_target.side_effect = none
```

## Routing Boundary

```text
routing_mode = evidence_only
external_paging_service_connected = false
automatic_remediation_allowed = false
automatic_submit_allowed = false
automatic_cancel_allowed = false
automatic_retry_allowed = false
automatic_replace_allowed = false
automatic_amend_allowed = false
automatic_flatten_allowed = false
live_exchange_request_allowed = false
adapter_send_allowed = false
```

## Golden Trace Coverage

```text
read_model.alert_taxonomy_routing.valid_matrix.001 = five covered alert categories route read-only
read_model.alert_taxonomy_routing.missing_required_fail_closed.001 = missing required fields fail closed
read_model.alert_taxonomy_routing.redaction_secret_fail_closed.001 = secret/signed/raw payload leak fails closed
read_model.alert_taxonomy_routing.automatic_action_fail_closed.001 = automatic action or remediation boundary fails closed
```

## Evidence

Validation is recorded in `docs/rust-cutover/evidence/V250-002.md` and
`verification.md`. The release replay manifest records the V250 alert taxonomy
cases as `validator_executable_replay`; this is alert evidence validation, not
runtime adapter integration.
