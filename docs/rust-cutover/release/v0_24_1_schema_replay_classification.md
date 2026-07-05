# v0.24.1 Schema Replay Classification

Date: 2026-07-05
Executor: Codex
Task: V241-004
GitHub issue: #773

## Summary

V241-004 classifies the v0.24.0 order-control golden traces that were
previously `schema_only_scoped`. The current classification is:

```text
validator_executable_replay = 39
schema_only_scoped v240 rows = 0
runtime adapter integration = 0
complete executable order-control runtime = false
production order submission allowed = false
execution adapter call allowed = false
Dashboard operation controls enabled = false
```

schema-only traces are not executable runtime. `validator_executable_replay`
means the trace is checked by an executable validator for its deterministic
decision envelope and fail-closed flags. It does not mean Rust runtime replay,
adapter integration, exchange truth, or live trading capability.

## Status Taxonomy

`schema_only_scoped`: schema and contract presence only. This status cannot be
used as executable order-control runtime evidence.

`validator_executable_replay`: executable validator replay over golden trace
decision envelopes. This proves the artifact envelope and boundary flags are
internally consistent, but it does not prove runtime adapter integration.

`executable_replay`: Rust harness replay through a crate or CLI test entrypoint.
This is stronger than validator replay and remains distinct.

`runtime_adapter_integration`: future adapter/runtime evidence. This class is
not present in v0.24.1.

## Promoted v240 Trace Groups

- `tests/golden/v240_order_intent_execution_policy.jsonl`: 4 decision rows.
- `tests/golden/v240_rate_limit_throttle_gate.jsonl`: 6 decision rows.
- `tests/golden/v240_order_slicing_preview.jsonl`: 6 decision rows.
- `tests/golden/v240_cancel_replace_amend_preview.jsonl`: 7 decision rows.
- `tests/golden/v240_retry_policy_ledger.jsonl`: 8 decision rows.
- `tests/golden/v240_readback_audit_evidence.jsonl`: 8 decision rows.

## Boundary

No runtime trading behavior changes. The validator explicitly keeps these flags
false in the release scope manifest and in every v240 decision envelope where
the field is present:

- `new_submit_capability`
- `production_order_mutation_allowed`
- `execution_adapter_call_allowed`
- `dashboard_operation_controls_enabled`
- `live_exchange_request_allowed`
- `network_attempted`
- `retry_scheduler_enabled`
- `implicit_retry_allowed`
- `cancel_replace_amend_send_allowed`
- `flatten_allowed`
- `child_order_submission_allowed`
- `child_order_scheduler_enabled`
- `exchange_truth_claimed`

## Gate

`scripts/ai/verify_release.sh v24.1-schema-replay-classification` fails closed
if:

- any current v240 order-control row remains `schema_only_scoped`;
- any promoted row claims a `rust_entrypoint`;
- any promoted row omits `runtime_adapter_integration = false`;
- any promoted row has a boundary flag opened;
- the release manifest claims complete executable order-control runtime,
  production mutation, adapter calls, live exchange requests, retry scheduler,
  Dashboard operation controls, or product-grade live trading terminal status.
