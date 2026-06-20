# NTPRO v0.11.1 Production Shadow Manifest Contract

Date: 2026-06-20
Executor: Codex
Status: IMPLEMENTED OFFLINE CONTRACT

## Summary

`v0.11.1` defines `v0_11/manifest.json` for local production shadow artifacts.
The manifest groups the redacted account snapshot, shadow execution intents,
shadow portfolio snapshot, order lifecycle state, and reconciliation events into
one auditable offline artifact set.

Plain Chinese summary: 这份 manifest 就是 v0.11 production shadow 工件的目录和
校验清单。Dashboard 可以用它确认哪些文件存在、记录数是多少、checksum 是否匹配。
它只证明本地 shadow 证据完整，不代表已经联网读取生产环境，也不代表可以实盘下单。

## Artifact Location

The manifest lives at:

```text
v0_11/manifest.json
```

The schema is:

```text
schema_version=ntpro.v111_production_shadow_manifest.v1
```

## Required Top-Level Fields

```text
schema_version
run_id
generated_at
artifact_root=v0_11
artifact_count
artifacts
summary
```

`generated_at` must be an ISO-8601 timestamp string. `artifact_count` must equal
the number of entries in `artifacts`.

## Artifact Entries

Every entry in `artifacts[]` must include:

```text
name
path
format=json|jsonl
required=true|false
present=true|false
record_count
byte_len
checksum=blake3:<hex>
raw_secret_recorded=false
raw_payload_recorded=false
signed_query_recorded=false
signed_url_recorded=false
```

Paths are relative to `v0_11/`. Absolute paths and `..` parent traversal are not
valid manifest artifact paths.

## Required Artifacts

The default required v0.11 production shadow artifact set is:

| name | path | format | required |
| --- | --- | --- | --- |
| `account_snapshot` | `account_snapshot_redacted.json` | `json` | yes |
| `shadow_execution_intent` | `shadow_execution_intent.jsonl` | `jsonl` | yes |
| `shadow_portfolio_snapshot` | `shadow_portfolio_snapshot.json` | `json` | yes |
| `order_lifecycle_state` | `order_lifecycle_state.jsonl` | `jsonl` | yes |
| `reconciliation_events` | `reconciliation_events.jsonl` | `jsonl` | yes |

Additional optional artifacts may be added later, but they must use the same
redaction and no-mutation fields.

## Summary Fields

The manifest `summary` must include:

```text
account_snapshots
shadow_intents_created
shadow_portfolio_snapshots_created
lifecycle_events_created
reconciliation_events_created
actual_submission_count=0
production_orders_submitted=0
production_order_mutations_attempted=0
dashboard_order_controls_enabled=false
raw_secret_recorded=false
raw_payload_recorded=false
```

## Dashboard Audit

Dashboard reads `v0_11/manifest.json` when present and treats it as read-only
evidence. The Dashboard audit checks:

- schema version;
- `generated_at`;
- declared `artifact_count`;
- child artifact existence for required entries;
- child artifact checksum;
- child artifact record count;
- forbidden raw secret / raw payload / signed query / signed URL flags;
- no production order submission or mutation counters;
- no Dashboard order controls.

If the manifest is absent, existing v0.11 artifact display remains backward
compatible and the manifest status is unknown. If the manifest is present but
invalid, Dashboard marks the Production Shadow section as degraded.

## Forbidden Behavior

The manifest must not:

- store API keys, API secrets, signatures, signed queries, or signed URLs;
- store raw production account payloads;
- authorize production order submission;
- authorize production cancel, replace, amend, retry, or correction;
- enable Dashboard order controls;
- claim successful online production reads;
- claim production trading readiness.

## Release Boundary

This contract is v0.11.1 hardening for local artifact accounting only. It does
not expand v0.11.0 capability and must not be presented as production network
read runtime or production trading support.
