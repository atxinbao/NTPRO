# NTPRO v0.12.0 Production Online Read-Only + Persistent Shadow Boundary

Date: 2026-06-20
Executor: Codex
Milestone: `v0.12.0`
Status: DESIGN GATE
Risk: high

## Summary

`v0.12.0` advances the v0.11 Production Read-Only Contract + Offline Shadow
Portfolio line into a narrower production-online track: owner-gated production
`GET` read-only probes and persistent local shadow artifact evidence.

Plain Chinese summary: v0.12.0 的目标不是“实盘交易”。它只允许在用户明确打开 gate
后，对生产 Binance 做只读 `GET` 证明，并把本地 shadow 组合、shadow 策略会话和
reconciliation 证据持久化。生产下单、撤单、改单、纠错单、真实资金变动、Dashboard
下单按钮都不属于 v0.12.0。

## Product Claim

`v0.12.0` may claim only:

- owner-gated production public read-only online proof;
- owner-gated authenticated production account snapshot online proof;
- validated read-only response shape evidence;
- persistent shadow portfolio runtime artifacts;
- bounded shadow strategy session JSONL event artifacts;
- read-only reconciliation classifications;
- Dashboard read-only status for production shadow artifacts.

`v0.12.0` must not claim:

- production order submission;
- production cancel, replace, amend, retry, or correction orders;
- production open-order or order-state read support;
- production trading readiness;
- real-funds mutation;
- strategy-driven production execution;
- Dashboard order controls;
- automatic production remediation;
- guarded live alpha order mutation.

## Version Sequence

```text
v0.11.0 = Production Read-Only Contract + Offline Shadow Portfolio
v0.11.1 = Production Read-Only Contract hardening patch candidate
v0.12.0 = Production Online Read-Only + Persistent Shadow
v0.13.0 = Earliest possible Guarded Live Alpha candidate
```

`v0.12.0` replaces the earlier planning shorthand that described v0.12.0 as
Guarded Live Alpha. Guarded Live Alpha is not part of this boundary.

## Default Execution Posture

Default local, PR, CI, and release-gate execution must remain offline and
read-only:

```text
production_public_online_read_probe = disabled
production_authenticated_online_read_probe = disabled
production_order_submission = forbidden
production_order_mutation = forbidden
production_open_order_reads = forbidden
dashboard_order_controls = false
shadow_runtime = local_artifact_only
```

Any missing owner gate, missing credential, unsupported endpoint, response-shape
mismatch, redaction failure, order endpoint access, listenKey lifecycle access,
or read/write ambiguity must fail closed before opening a production request.

## Allowed Production Online Read-Only Surfaces

v0.12.0 may implement only the following owner-gated production online paths:

| Method | Path | Purpose | Allowed artifact output |
| --- | --- | --- | --- |
| `GET` | `/api/v3/time` | Public production server-time proof. | status class, latency, schema summary, redacted URL, `serverTime` shape validation. |
| `GET` | `/api/v3/exchangeInfo` | Public production exchange metadata proof. | status class, latency, bounded schema summary, redacted URL. |
| `GET` | `/api/v3/account` | Authenticated production account snapshot proof. | status class, latency, bounded redacted account shape summary. |

Constraints:

- The host must be production Binance spot only.
- The request method must be `GET`.
- Online execution must require explicit CLI and environment owner gates.
- Authenticated signing must happen in memory only.
- Artifacts may record credential presence booleans, never credential values.
- Account snapshot artifacts must not record raw balances, asset symbols,
  account identifiers, permissions, commission details, raw headers, raw
  response payloads, signatures, signed query strings, or signed URLs.

If an endpoint is not explicitly listed above, it is denied.

## Explicitly Forbidden Surfaces

The following are forbidden in v0.12.0:

| Method / class | Path / class | Reason |
| --- | --- | --- |
| `POST` | `/api/v3/order` | Production order submission. |
| `POST` | `/api/v3/order/test` | Production order-submit surface, even if exchange matching is bypassed. |
| `DELETE` | `/api/v3/order` | Production order cancellation. |
| `PUT` / `PATCH` | any production order endpoint | Production order replacement or mutation. |
| any method | `/api/v3/openOrders`, `/api/v3/allOrders`, `/api/v3/order` | Production order-state reads remain out of scope. |
| any method | `/api/v3/orderList`, `/api/v3/openOrderList`, `/api/v3/allOrderList` | Production order-list reads remain out of scope. |
| `POST` / `PUT` / `DELETE` | `/api/v3/userDataStream` | listenKey lifecycle mutation. |
| signed WebSocket user stream | listenKey-backed user data stream | Deferred for v0.12 because listenKey lifecycle access is out of scope. |
| any method | margin, futures, transfer, withdrawal, deposit, account settings | Expanded account or funds surface. |
| any Dashboard action | order/cancel/replace/amend/retry/reconnect/credential controls | Dashboard remains read-only for v0.12.0. |

## Owner Gate Contract

Production online read-only attempts may occur only when every required gate is
true for the specific probe:

```text
explicit v0.12 manual-online mode is enabled
explicit production network permission is enabled
explicit production read-only CLI flag is passed
configured environment is production
request method is GET
request target is allowlisted
order_submission = disabled
order_mutation = disabled
dashboard_order_controls = false
credential values are env-only and never persisted
```

Default runs must produce blocked/offline artifacts with:

```text
network_attempted = false
account_read_attempted = false
production_order_submission_attempted = false
production_order_mutation_attempted = false
dashboard_order_controls_enabled = false
```

Manual online runs may set `network_attempted=true` only after all owner gates
pass and an allowlisted `GET` request is actually attempted.

Authenticated account proof may set `account_read_attempted=true` only for the
owner-gated `/api/v3/account` production snapshot path.

## Artifact Boundary

v0.12 artifacts may include:

```text
v0_12/production_public_online_read_probe.json
v0_12/production_account_snapshot_redacted.json
v0_12/production_readonly_response_shape.json
v0_12/shadow_portfolio_runtime.json
v0_12/shadow_strategy_session.jsonl
v0_12/reconciliation_events.jsonl
v0_12/dashboard_production_shadow_readonly_status.json
v0_12/summary.json
```

Required summary counters:

```text
production_public_online_reads_attempted
production_authenticated_account_reads_attempted
production_order_submissions_attempted
production_order_mutations_attempted
production_order_state_reads_attempted
listen_key_lifecycle_attempted
shadow_portfolio_snapshots_created
shadow_strategy_sessions_started
dashboard_order_controls_enabled
redaction_passed
```

Required invariant values:

```text
production_order_submissions_attempted = 0
production_order_mutations_attempted = 0
production_order_state_reads_attempted = 0
listen_key_lifecycle_attempted = 0
dashboard_order_controls_enabled = false
```

## Persistent Shadow Artifact Boundary

Persistent shadow artifacts may record local state only:

- shadow portfolio snapshots;
- bounded shadow strategy session heartbeat and stop event markers;
- read-only reconciliation classifications;
- artifact provenance, schema version, and generated timestamp;
- local risk-halt classifications.

Persistent shadow artifacts must not:

- submit production orders;
- cancel, replace, amend, retry, or correct production orders;
- convert reconciliation mismatch into an exchange-side action;
- present shadow fills, shadow positions, or derived exposure as exchange truth;
- persist raw credentials, raw account response bodies, raw balances, raw
  signatures, signed URLs, signed query strings, or raw headers.

## Dashboard Boundary

Dashboard may display read-only production shadow status only:

- public read-only probe status;
- authenticated account snapshot status;
- response-shape validation status;
- shadow portfolio runtime status;
- bounded shadow strategy session heartbeat event;
- reconciliation classification;
- artifact path and freshness.

Dashboard must not:

- start production probes;
- load or render credential values;
- render raw account responses or raw balances;
- submit, cancel, replace, amend, retry, reconnect, or correct orders;
- expose credential input fields;
- present shadow data as exchange-confirmed positions or fills.

## Failure Posture

v0.12.0 must fail closed when:

- an owner gate is missing;
- the target endpoint is not allowlisted;
- the method is not `GET`;
- the response shape is incompatible or ambiguous;
- redaction cannot be proven;
- credential values would be logged or persisted;
- order endpoint access is detected;
- listenKey lifecycle access is detected;
- Dashboard action controls are present.

Failure artifacts may record bounded diagnostics and counters only. They must
not contain secrets, raw account payloads, raw balances, signed URLs, signatures,
or exchange mutation requests.

## Task Ladder

v0.12 work must proceed in this order:

1. `V120-000` - Define production online read-only boundary.
2. `V120-001` - Add production public read-only online probe.
3. `V120-002` - Add authenticated production account snapshot online proof.
4. `V120-003` - Validate production read-only response shape.
5. `V120-004` - Implement shadow portfolio runtime.
6. `V120-005` - Add bounded shadow strategy session event artifact.
7. `V120-006` - Add production read-only reconciliation engine.
8. `V120-007` - Add Dashboard production shadow read-only panel.
9. `V120-008` - Add dual release gates for offline and owner-gated online.
10. `V120-009` - Prepare v0.12 readiness and release notes.

Each high-risk task must stop at `REVIEW_REQUIRED` and must not enable
auto-merge unless a later owner-approved scope decision changes the risk
protocol.

## Release Rule

`ntpro-rust-only-v0.12.0` may be considered only after all V120 tasks record
evidence that:

- production online reads were owner-gated and `GET`-only;
- production order submissions stayed `0`;
- production order mutations stayed `0`;
- production order-state reads stayed `0`;
- listenKey lifecycle mutation stayed `0`;
- Dashboard order controls stayed absent;
- raw secrets, raw signatures, signed URLs, raw account responses, and raw
  balance values were not persisted;
- persistent shadow artifacts did not present local shadow state as exchange truth.
