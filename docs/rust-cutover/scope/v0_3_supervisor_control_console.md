# NTPRO v0.3.0 Scope Addendum - Local Supervisor Control Console

Date: 2026-06-12
Executor: Codex
Status: proposed release blocker

## Decision

The existing v0.3 Dashboard MVP proves a local system-status cockpit with
start/stop controls. Before publishing `ntpro-rust-only-v0.3.0`, the release
claim should be upgraded to:

```text
Local Supervisor Control Console
```

Plain language:

```text
v0.3.0 should not only show local sandbox node state.
It should also prove the local supervisor can perform the basic non-trading
control actions operators expect from a local console.
```

## Required v0.3.0 Controls

The v0.3.0 release gate must prove these local sandbox-only controls:

```text
start node
stop node
pause node
resume node
reconnect data source
reconnect execution gateway
```

The current Dashboard MVP already covers `start` and `stop`. The following
actions remain unsupported today and become v0.3.0 release blockers if the
release name claims a local supervisor control console:

```text
pause node
resume node
reconnect data source
reconnect execution gateway
```

## Control Semantics

Pause and resume must be application-level supervisor controls, not operating
system `SIGSTOP` / `SIGCONT` controls.

Pause means:

- the node process stays alive;
- heartbeat, status, metrics, logs, and stop control remain responsive;
- lifecycle state becomes `Paused` or an equivalent explicit paused state;
- strategy/execution progress is disabled or held by a documented local
  sandbox-only control path;
- no real orders are submitted.

Resume means:

- the node transitions from paused back to running;
- status and metrics record the transition;
- the node remains sandbox-only;
- no real orders are submitted.

Reconnect data source means:

- the control is local and sandbox/mock/fixture-only in v0.3.0;
- the data connection state transition is observable in status or metrics;
- unavailable real adapter reconnect must return a clear unsupported result.

Reconnect execution gateway means:

- the control is local and sandbox/mock/fixture-only in v0.3.0;
- the execution gateway connection state transition is observable in status or
  metrics;
- unavailable real adapter reconnect must return a clear unsupported result.

## Out Of Scope

The upgraded v0.3.0 release still must not claim:

- production real-exchange live trading;
- real account connectivity;
- real order submission;
- manual order entry;
- order modification;
- order cancellation;
- strategy parameter hot reload;
- production reconnect controls;
- remote or distributed dashboard operation;
- multi-user permissions;
- prebuilt binary or Docker delivery.

## Required Release Gate

Add a dedicated local smoke gate:

```bash
scripts/ai/v03_supervisor_control_smoke.sh
```

The smoke must prove:

```text
register two sandbox nodes
start node
pause node
status == paused
resume node
status == running
reconnect data source
data reconnect status == ok or explicit sandbox unsupported result
reconnect execution gateway
execution reconnect status == ok or explicit sandbox unsupported result
stop node
real_orders_submitted=false
external_venue_connection=false
```

`scripts/ai/verify_release.sh` must run this smoke before a v0.3.0 tag or
GitHub Release is approved.

## Task Sequence

```text
V03-010 Dashboard smoke and v0.3 readiness report
  -> V03-011 v0.3.0 supervisor control release contract
  -> V03-012 pause/resume node control
  -> V03-013 reconnect data source control
  -> V03-014 reconnect execution gateway control
  -> V03-015 Dashboard controls and API smoke
  -> V03-016 v0.3.0 release gate and readiness report
```

## Release Decision

Do not publish `ntpro-rust-only-v0.3.0` until `V03-011` through `V03-016`
have evidence and the final readiness report records strict PASS.

