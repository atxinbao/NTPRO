# NTPRO v0.3 Scope Decision - Dashboard MVP

Date: 2026-06-07
Executor: Codex
Status: active scope decision

## Decision

NTPRO v0.3 is scoped as:

```text
Dashboard MVP - Local System Status Cockpit
```

The v0.3 goal is to make the v0.2 local multi-node runtime visible and lightly
controllable from a local dashboard.

Plain language:

```text
Open one local page.
See whether NTPRO is alive.
See which nodes are running or stopped.
See data source, execution gateway, risk engine, runtime module, logs, and
metrics state.
Start or stop a local sandbox node.
```

v0.3 is not a trading terminal and not a full Pro control plane.

## Baseline

v0.3 starts after the formal `ntpro-rust-only-v0.2.0` release.

The required v0.2 input is:

- local `ntpro-node` process path;
- local supervisor registry;
- supervisor start, stop, status, connections, execution, risk, logs, and
  metrics commands;
- `NodeStatus` and `NodeMetrics` artifacts;
- two-node local sandbox smoke evidence;
- `docs/rust-cutover/release/v0_2_local_multi_node_readiness_report.md`.

## In Scope

- Define the Dashboard read model as `DashboardSnapshot`.
- Define dashboard-readable status details for:
  - overview;
  - nodes;
  - data sources;
  - execution gateways;
  - risk engine;
  - runtime modules;
  - logs;
  - metrics;
  - alerts;
  - gaps;
  - controls.
- Aggregate dashboard state from v0.2 supervisor artifacts:
  - `registry.json`;
  - `status.json`;
  - `metrics.json`;
  - `logs/events.log`;
  - `logs/stdout.log`;
  - `logs/stderr.log`.
- Add a local dashboard HTTP server.
- Serve static HTML, CSS, and JavaScript without a frontend framework.
- Render:
  - Overview;
  - Nodes;
  - Data Sources;
  - Execution;
  - Risk;
  - Logs / Metrics;
  - Runtime Modules.
- Add local start and stop lifecycle controls.
- Show pause, resume, reconnect, and other unsupported controls as disabled or
  unsupported.
- Prove the MVP with a two-node dashboard smoke and browser verification.
- Produce a final v0.3 readiness report with strict PASS/FAIL.

## Out Of Scope

- Production real-exchange live trading.
- Manual order entry.
- Order modification.
- Order cancellation.
- VWAP, POV, Iceberg, or other execution algorithm productization.
- Strategy parameter hot reload.
- Multi-user permissions.
- Remote or distributed multi-server supervisor operation.
- Nexus-like high-performance message bus.
- Docker delivery as a v0.3 acceptance requirement.
- React, Vue, Next.js, Vite, or another frontend build system.
- Reading mutable `LiveNode`, engine, cache, message bus, adapter, or account
  internals directly from the dashboard.
- Tag creation or GitHub Release publication.

## Product Boundary

Dashboard data must flow through a read model:

```text
SupervisorRegistry
  + NodeStatus
  + NodeMetrics
  + local log artifacts
  -> DashboardSnapshot
  -> local HTTP API
  -> static Dashboard UI
```

The dashboard must not directly mutate trading internals. It may request
lifecycle actions through the local supervisor path only.

## Required Panels

| Panel | Required state | Plain language |
| --- | --- | --- |
| Overview | Node counts, health summary, sandbox-only flags, latest transition, latest error. | Is the system alive? |
| Nodes | `node_id`, lifecycle, process, pid, config path, artifact root, timestamps, last error. | What is each node doing? |
| Data Sources | Source alias, provider, connection, freshness, health, last error. | Is data coming in? |
| Execution | Gateway alias, venue, connection, started, redacted account ref, order count summary, last error. | Is the execution gateway up? |
| Risk | Trading state, health, rejection count, last rejection, last error. | Is risk allowing or blocking trading? |
| Logs / Metrics | Log paths, lifecycle events, uptime, start/stop counters, state transitions. | Where do we inspect evidence? |
| Runtime Modules | LiveNode, NautilusKernel, DataEngine, ExecutionEngine, RiskEngine, Portfolio, Cache, MessageBus, Logging, Metrics writer, Supervisor. | Which internal modules look healthy, unknown, or unsupported? |

Missing data must be explicit:

```text
unknown
not_configured
not_supported
stale
redacted
```

Dashboard code must not silently report missing data as healthy.

## Control Policy

v0.3 may implement:

```text
start
stop
```

v0.3 must not implement:

```text
manual order entry
order modification
order cancellation
strategy hot reload
production reconnect controls
```

Pause, resume, reconnect data, and reconnect execution may appear only as
disabled or unsupported controls with a clear reason.

## Task Sequence

```text
V03-001 Scope decision / Dashboard system-status MVP
  -> V03-002 DashboardSnapshot DTO
  -> V03-003 Status detail DTOs
  -> V03-004 Supervisor artifacts to DashboardSnapshot aggregator
  -> V03-005 Local dashboard HTTP server
  -> V03-006 Overview and Nodes UI
  -> V03-007 Data source / execution / risk panels
  -> V03-008 Runtime modules diagnostic panel
  -> V03-009 Dashboard start/stop controls
  -> V03-010 Dashboard smoke and v0.3 readiness report
```

## Validation Gates

The v0.3 readiness report may declare PASS only if the scoped evidence includes:

```bash
cargo fmt --check
cargo check -p nautilus-cli
cargo test -p nautilus-cli dashboard --lib
cargo test -p nautilus-cli supervisor --lib
scripts/ai/v02_two_node_supervisor_smoke.sh
scripts/ai/v03_dashboard_smoke.sh
scripts/ai/check_rust_only_runtime.sh
git diff --check
```

Browser or Playwright verification must also confirm:

- the dashboard page opens locally;
- the page renders real supervisor/node state;
- two sandbox nodes are visible;
- data source, execution, risk, runtime modules, logs, and metrics panels render;
- start/stop button states are correct;
- unsupported controls are disabled;
- visible text and controls do not overlap at desktop and mobile widths.

## Release Boundary

V03 completion is not release approval. `V03-010` may declare Dashboard MVP
readiness only after it cites evidence for `V03-001` through `V03-009`; tag
creation or GitHub Release publication still requires separate explicit user
approval.
