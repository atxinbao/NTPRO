# Local Supervisor Pause/Resume Contract

Date: 2026-06-12
Executor: Codex

## Purpose

This document defines the current `pause` and `resume` semantics for the
NTPRO local Supervisor control console.

Plain Chinese summary:

```text
pause/resume 现在只是本地 supervisor 的 artifact 状态控制。
它不会冻结操作系统进程，不会暂停真实交易 runtime，也不会控制真实交易所、
adapter、策略循环或执行循环。
```

## Product Boundary

The current scope is local sandbox Supervisor control. The control surface reads
and writes local node artifacts such as registry, status, metrics, events, and
PID artifacts.

This boundary means:

- the source of truth is local filesystem artifact state;
- `pause` and `resume` are local supervisor lifecycle markers;
- the supervised process remains a normal running process while paused;
- external venue connectivity remains disabled;
- real order submission remains disabled;
- production trading runtime suspension is not implemented by this contract.

## What `pause` Does

`pause` is accepted only for a node whose supervisor record can be refreshed and
whose current local lifecycle state is `running`.

On success, the supervisor:

- keeps the process state as `running`;
- writes the local status artifact lifecycle state as `paused`;
- records the previous lifecycle state as `running`;
- updates local metrics, including lifecycle transition counters;
- appends a local supervisor event for the pause action;
- keeps `external_venue_connection=false`;
- keeps `real_orders_submitted=false`.

`pause` does not send `SIGSTOP`, does not freeze the process, and does not stop
the node. A later `stop` action remains valid while the node is marked paused.

## What `resume` Does

`resume` is accepted only for a node whose local lifecycle state is `paused`.

On success, the supervisor:

- keeps the process state as `running`;
- writes the local status artifact lifecycle state as `running`;
- records the previous lifecycle state as `paused`;
- updates local metrics and events for the resume action;
- keeps `external_venue_connection=false`;
- keeps `real_orders_submitted=false`.

`resume` does not send `SIGCONT`. It also does not reconnect any adapter, restart
strategy execution, or replay runtime work. It only returns the local supervisor
lifecycle marker from `paused` to `running`.

## Observable And Controllable While Paused

While a node is marked paused, operators can still inspect local artifacts:

- registry state;
- status artifact;
- metrics artifact;
- stdout/stderr log artifacts;
- events artifact.

The local `stop` action remains supported while paused. Local reconnect actions
remain explicit sandbox `not_supported` results; they are not production adapter
reconnect controls.

## Unsupported Interpretations

The following interpretations are explicitly unsupported:

- `pause` as OS-level suspension with `SIGSTOP`;
- `resume` as OS-level continuation with `SIGCONT`;
- `pause` as live trading runtime suspension;
- `resume` as live trading runtime restart;
- adapter suspension or adapter reconnect;
- strategy-loop suspension;
- execution-loop suspension;
- real order halt, cancel, reject, throttle, or gateway-level behavior;
- production real-exchange live trading control.

If NTPRO later adds production runtime pause/resume, it must be a separate
runtime contract with tests, release evidence, and migration notes. That future
work must not be inferred from this local artifact contract.

## Operator Guidance

Treat `paused` as a local control-console marker, not as a production risk
control.

For current v0.3.1 behavior:

- use `pause` to mark a local sandbox supervisor node as paused;
- use `resume` to mark that same local node as running again;
- use `stop` to terminate the local sandbox process;
- do not use this surface as a real trading kill switch;
- do not assume any real adapter or order flow is affected.

## Evidence Anchors

- `docs/rust-cutover/evidence/V03-012.md`
- `docs/rust-cutover/scope/v0_3_1_supervisor_control_hardening.md`
- `crates/cli/src/supervisor.rs`
