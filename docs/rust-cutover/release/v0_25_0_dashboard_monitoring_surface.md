# v0.25.0 Dashboard monitoring surface contract

Date: 2026-07-05
Executor: Codex
Task: `V250-006`
GitHub issue: `#783`

## Summary

The v0.25.0 Dashboard / Trader Terminal surface is a read-only observer for
monitoring, alert, incident, runbook/audit, and DR preview evidence. It consumes
the existing unified read-model artifact and exposes v25 status fields in the
Workbench and Unified Read Model table.

Plain Chinese summary: v0.25.0 Dashboard 只展示监控、告警、incident、runbook 和
DR preview 证据。它不提供任何交易按钮，不发 live request，不调用 adapter send，不执行
runbook 或 DR restore，也不声明自己是产品级实盘终端。

## Runtime boundary

Every v25 Dashboard surface component must provide:

- component status;
- source provenance;
- freshness;
- redaction state;
- read-only operation boundary;
- no forbidden trading-control marker.

The surface fails closed when it sees any of these markers as `true`:

- `submit_order_allowed`;
- `cancel_order_allowed`;
- `retry_order_allowed`;
- `replace_order_allowed`;
- `amend_order_allowed`;
- `flatten_position_allowed`;
- `order_ticket_enabled`;
- `dashboard_trading_control_allowed`;
- `live_exchange_request_allowed`;
- `adapter_send_allowed`;
- `automatic_remediation_allowed`.

Missing v25 surface components, stale freshness, or degraded component status
cannot display healthy. Missing provenance or redaction fails closed.

## Dashboard scope

Included:

- monitoring effective status and source ref;
- alert severity, route status, and dedupe key;
- incident state, owner, and acknowledgement status;
- runbook decision type/status and evidence ref;
- DR preview scenario, recovery point, operator approval, and snapshot lineage;
- v25 blocking reasons for provenance/freshness/redaction/boundary diagnostics.

Excluded:

- submit/cancel/retry/replace/amend/flatten controls;
- order ticket;
- live control API;
- production order mutation;
- adapter send;
- live exchange request;
- automatic remediation or retry scheduler.

## Validation contract

`scripts/ai/verify_release.sh v25-dashboard-monitoring-surface` validates the
golden trace decision envelope and checks the Dashboard source keeps the v25
surface markers while not exposing forbidden workbench/order actions.

## Post-Release Source Ref Integrity

Date: 2026-07-06
Executor: Codex
Task: `V251-004` / GitHub issue `#809`

`scripts/ai/verify_release.sh v25.1-dashboard-source-ref-integrity` hardens the
same v25 Dashboard surface gate by resolving component `source_ref` values.
The gate validates repository-relative paths, JSONL case anchors, Markdown
heading anchors, and v25 release contract refs for monitoring, alert, incident,
runbook, and DR preview components.

```text
source_refs_resolved = 24
release_contract_refs = 5
bad_path_selftest = fail_closed
bad_jsonl_anchor_selftest = fail_closed
bad_markdown_anchor_selftest = fail_closed
empty_source_ref_selftest = fail_closed
cross_version_ref_selftest = fail_closed
```

This post-release hardening changes validation only. Dashboard remains
read-only and still has no submit, cancel, retry, replace, amend, flatten,
order-ticket, adapter send, live exchange request, automatic remediation, or
production order mutation path.
