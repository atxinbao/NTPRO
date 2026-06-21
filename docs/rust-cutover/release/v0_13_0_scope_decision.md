# NTPRO v0.13.0 Guarded Live Alpha Preflight Scope Decision

Date: 2026-06-21
Executor: Codex
Milestone: `v0.13.0`
Status: SCOPE DECISION
Risk: medium

## Summary

`v0.13.0` is scoped as Guarded Live Alpha Preflight only. It is not a production
order submission, production order mutation, real-funds, or production trading
release.

Plain Chinese summary: v0.13.0 不是“开始实盘下单”。它只是把未来进入 live alpha
之前必须具备的证据链先搭起来：长时间 shadow session、owner 手动只读证明、kill
switch 干跑、手动批准记录、Decimal/string 金额边界、Dashboard 角色边界，以及默认不
触发生产订单 mutation 的 release gate。

## Product Claim

`v0.13.0` may claim only:

- Guarded Live Alpha Preflight scope control;
- local long-running shadow session evidence;
- owner-run production read-only proof pack evidence;
- kill-switch dry-run and manual approval artifacts;
- trader/ops Dashboard control boundary documentation;
- Decimal/string-only risk and execution amount boundary;
- no-production-mutation release gate evidence.

`v0.13.0` must not claim:

- production order submission;
- production cancel, replace, amend, retry, correction, or reconnect actions;
- production order-state read readiness;
- listenKey lifecycle readiness;
- signed WebSocket user stream runtime readiness;
- strategy-driven production execution;
- automatic production remediation;
- production portfolio parity;
- live-alpha risk/execution-grade money math;
- real funds;
- production trading;
- Dashboard order controls.

## Version Sequence

```text
v0.12.0 = Production Online Read-Only + Persistent Shadow
v0.12.1 = Production Read-Only Evidence & Release Surface Hardening
v0.13.0 = Guarded Live Alpha Preflight only
```

The phrase "Guarded Live Alpha" in the v0.13 line means preflight evidence for
a future owner-approved alpha path. It does not mean production order mutation
is enabled by v0.13.0.

## Default Execution Posture

Default local, PR, CI, and release-gate execution must remain fail-closed and
no-mutation:

```text
production_order_submission = forbidden
production_order_mutation = forbidden
production_order_state_reads = forbidden unless separately owner-approved
listen_key_lifecycle = forbidden unless separately owner-approved
dashboard_order_controls = false
manual_approval_artifact = preflight_only
kill_switch = dry_run_only
shadow_session = local_artifact_only
```

## V130 Task Ladder

V130 work must proceed in this order:

1. `V130-001` - Guarded Live Alpha Preflight scope decision.
2. `V130-002` - Long-running shadow session process with heartbeat, stop, and
   stale-data handling.
3. `V130-003` - Owner-run production online read-only proof pack.
4. `V130-004` - Kill switch dry-run and manual approval artifact contract.
5. `V130-005` - Trader/Ops Dashboard control boundary.
6. `V130-006` - Decimal-only risk/execution amount boundary.
7. `V130-007` - No-production-mutation release gate for live-alpha preflight.

## Required Evidence Before Any Later Mutation Release

Any later version that proposes production order mutation must have a separate
owner-approved scope decision and must first prove:

- exact venue, environment, account boundary, symbol set, order type, notional
  or quantity limit, and stop conditions;
- no-mutation default gate for local, PR, CI, and release runs;
- kill-switch dry-run artifact and manual approval artifact;
- long-running shadow session heartbeat, stop, stale-data, and artifact-gap
  behavior;
- owner-run production read-only proof pack with redaction;
- Decimal/string-only amount handling for all risk and execution preflight
  fields;
- Dashboard role boundary that keeps trader-visible order controls disabled;
- risk rejection, reconciliation, and golden trace evidence for the proposed
  mutation path;
- rollback and incident evidence for the proposed mutation path.

## Explicit Non-Goals

This scope decision does not create a production live alpha. It does not create
a production order path. It does not authorize any CLI, Dashboard, supervisor,
node, strategy, adapter, risk, or execution code to submit, cancel, replace,
amend, retry, correct, reconnect, or otherwise mutate production exchange
orders.
