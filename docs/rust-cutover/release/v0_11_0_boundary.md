# NTPRO v0.11.0 Production Read-Only + Shadow Portfolio Boundary

Date: 2026-06-19
Executor: Codex
Status: RELEASED BOUNDARY

## Summary

`v0.11.0` is the published capability track after the `v0.10.0` Binance spot
sandbox order proof line. Its scope is Production Read-Only + Shadow Portfolio:
read production market/account state through explicit gates, create local shadow
intents and portfolio snapshots, and write evidence artifacts that prove the
system stayed read-only.

Plain Chinese summary: v0.11.0 只能“看生产环境”和“本地影子计算”，不能“动生产环境”。
允许的是只读读取、影子订单意图、影子组合快照、影子 reconciliation 事件；禁止的是
生产下单、撤单、改单、自动纠错单、真实资金变动和 Dashboard 下单按钮。

## Version Sequence

```text
v0.9.0  = Strategy Runtime Foundation
v0.10.0 = Binance Spot Sandbox Order Proof
v0.10.1 = Release-surface hotfix line
v0.11.0 = Production Read-Only + Shadow Portfolio
v0.12.0 = Earliest possible Guarded Live Alpha
```

`v0.11.0` must not be described as a production trading release. `v0.12.0` is
the earliest track that may discuss guarded live order mutation, and only after
separate owner approval, risk gates, and release evidence.

## Product Claim

`v0.11.0` may claim only:

- production endpoint classification with fail-closed defaults;
- public production read-only probe readiness;
- owner-gated authenticated production read-only account snapshot contract;
- local shadow execution intent artifacts;
- minimal shadow portfolio snapshots;
- shadow/read-only order lifecycle state model;
- production read-only reconciliation event model;
- read-only Dashboard status for shadow artifacts and read-only probes.

`v0.11.0` must not claim:

- production order submission;
- production cancel, replace, amend, retry, or correction orders;
- production trading readiness;
- real-funds mutation;
- Dashboard order controls;
- automatic online order mutation;
- production order lifecycle parity.

## Default Execution Posture

Default posture is offline and read-only:

```text
production_public_read_probe = disabled
production_authenticated_read_probe = disabled
production_order_mutation = forbidden
shadow_execution_intent = local_artifact_only
shadow_portfolio = local_artifact_only
dashboard_order_controls = false
```

Any missing owner gate, missing credential, endpoint classification mismatch,
secret-redaction failure, unsupported account state, reconciliation mismatch, or
read/write ambiguity must fail closed.

## Allowed Production Read-Only Surfaces

v0.11 may add release-scoped contracts for these read-only surfaces:

| Surface | Allowed behavior | Mutation allowed |
| --- | --- | --- |
| Endpoint classifier | Classify sandbox, production public read-only, production authenticated read-only, and forbidden mutation endpoints. | No |
| Public read-only probe | Read public market/server-time style data from production endpoints through explicit gates. | No |
| Authenticated account snapshot | Read account/balance/open-order snapshots through explicit owner gates and redacted artifacts. | No |
| Shadow execution intent | Build local intent records that explicitly carry `submission_allowed=false`. | No |
| Shadow portfolio | Build local portfolio snapshots from read-only inputs and local shadow state. | No |
| Reconciliation events | Record read-only reconciliation observations and risk-halt decisions. | No |
| Dashboard status | Display read-only status and artifact paths. | No |

## Forbidden Surfaces

The following are forbidden in v0.11:

- `POST`, `DELETE`, or mutation-style production Binance endpoints;
- production order submit/cancel/replace/amend/retry/correction;
- strategy-driven production execution;
- automated remediation that creates, cancels, or amends an exchange order;
- Dashboard controls that can create or mutate exchange orders;
- persisted API key, API secret, raw signature, signed URL, or signed query;
- wording that presents shadow artifacts as real production fills or positions.

## Artifact Boundary

v0.11 artifacts may include:

```text
v0_11/endpoint_classifier.json
v0_11/public_read_probe.json
v0_11/account_snapshot_redacted.json
v0_11/shadow_execution_intent.jsonl
v0_11/shadow_portfolio_snapshot.json
v0_11/order_lifecycle_state.jsonl
v0_11/reconciliation_events.jsonl
v0_11/dashboard_readonly_status.json
v0_11/summary.json
```

Required summary counters:

```text
production_public_reads_attempted
production_authenticated_reads_attempted
production_order_submissions_attempted
production_order_mutations_attempted
shadow_intents_created
shadow_portfolio_snapshots_created
dashboard_order_controls_enabled
redaction_passed
```

`production_order_submissions_attempted` and
`production_order_mutations_attempted` must stay `0`.
`dashboard_order_controls_enabled` must stay `false`.

## Gate Ladder

v0.11 work must pass this ladder in order:

1. **Boundary gate**: public docs state read-only/shadow-only scope.
2. **Endpoint classifier gate**: mutation endpoints are classified as forbidden.
3. **Public read-only gate**: public production reads require explicit local
   opt-in and redacted artifacts.
4. **Authenticated read-only gate**: account reads require owner approval,
   credentials, redaction, and no mutation endpoint access.
5. **Shadow intent gate**: local intents require `submission_allowed=false`.
6. **Shadow portfolio gate**: portfolio snapshots are computed locally and must
   not be presented as exchange positions.
7. **Reconciliation gate**: mismatches produce local risk-halt evidence, not
   correction orders.
8. **Dashboard read-only gate**: Dashboard may show status and artifact paths
   only; it must not initiate or cancel orders.

## Release Rule

The `ntpro-rust-only-v0.11.0` tag may be published only after every v0.11 task
records evidence that production order mutation stayed disabled and Dashboard
order controls stayed absent.
