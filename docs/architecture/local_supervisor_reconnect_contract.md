# Local Supervisor Reconnect Contract

Date: 2026-06-12
Executor: Codex

## Purpose

This document defines the current `reconnect-data` and `reconnect-execution`
semantics for the NTPRO local Supervisor control console.

Plain Chinese summary:

```text
当前 reconnect 控制只会在本地 sandbox supervisor 中记录 not_supported。
它不会连接真实交易所，不会重连真实 adapter，也不会恢复真实订单或账户通道。
```

## Product Boundary

The current reconnect surface is local sandbox-only. It exists so the CLI,
Dashboard API, Dashboard UI, status artifacts, metrics artifacts, and events
artifacts can report the same honest result when an operator asks for reconnect
inside the local Supervisor scope.

This boundary means:

- the action is accepted only for a local registered node;
- the node process must be running;
- the local lifecycle state must be `running` or `paused`;
- the result is always `not_supported` for the current sandbox path;
- no real venue endpoint is contacted;
- no production data adapter or execution adapter is reconnected;
- no real orders are submitted, modified, cancelled, or replayed.

## `reconnect-data`

`reconnect-data` records a data-source reconnect request as unsupported for the
current local sandbox path.

On success, the Supervisor:

- keeps the node lifecycle state as `running` or `paused`;
- writes `data_connection=not_supported` to the status artifact;
- writes the same unsupported summary to metrics;
- appends `phase=reconnect_data status=not_supported` to events;
- keeps `external_venue_connection=false`;
- keeps `real_orders_submitted=false`;
- reports the Dashboard API error code as `sandbox_reconnect_not_supported`.

It does not reconnect a real market-data venue, restore subscriptions, replay
market data, or touch live credentials.

## `reconnect-execution`

`reconnect-execution` records an execution-gateway reconnect request as
unsupported for the current local sandbox path.

On success, the Supervisor:

- keeps the node lifecycle state as `running` or `paused`;
- writes `execution_connection=not_supported` to the status artifact;
- writes `execution.connection=not_supported` in the execution detail status;
- writes the same unsupported summary to metrics;
- appends `phase=reconnect_execution status=not_supported` to events;
- keeps `external_venue_connection=false`;
- keeps `real_orders_submitted=false`;
- reports the Dashboard API error code as `sandbox_reconnect_not_supported`.

It does not reconnect a real execution gateway, reconcile a real account, place
orders, cancel orders, modify orders, or recover live order state.

## User-Facing Wording

Every user-facing surface should make the limitation visible:

| Surface | Required wording behavior |
| --- | --- |
| CLI | Print `status=not_supported` and a local sandbox reason. |
| Dashboard control button | Use wording that says the action records unsupported reconnect, not that it performs real reconnect. |
| Dashboard API | Return `status=not_supported` and `error_code=sandbox_reconnect_not_supported`. |
| Status artifact | Preserve `external_venue_connection=false` and `real_orders_submitted=false`. |
| Metrics artifact | Preserve the same no-real-venue and no-real-order markers. |
| Events artifact | Record `phase=reconnect_data status=not_supported` or `phase=reconnect_execution status=not_supported`. |

## Unsupported Interpretations

The following interpretations are explicitly unsupported:

- production venue reconnect;
- production data adapter reconnect;
- production execution adapter reconnect;
- real account session recovery;
- live subscription recovery;
- live execution reconciliation;
- real order submission, cancellation, modification, or replay;
- automatic recovery after network failure.

If NTPRO later adds production reconnect controls, that work must be a separate
runtime/adapter contract with fixture-backed tests, release evidence, and a
migration note. It must not be inferred from this local sandbox contract.

## Evidence Anchors

- `docs/rust-cutover/evidence/V03-013.md`
- `docs/rust-cutover/evidence/V03-014.md`
- `docs/rust-cutover/evidence/V03-015.md`
- `docs/rust-cutover/scope/v0_3_1_supervisor_control_hardening.md`
- `crates/cli/src/supervisor.rs`
- `crates/cli/src/dashboard.rs`
