# NTPRO Dashboard MVP Scope Contract

Date: 2026-06-04
Executor: Codex
Task ID: NDASH-001

## Purpose

This document locks the first NTPRO dashboard MVP scope before implementation
starts. It keeps dashboard work focused on local operator visibility and a small
set of lifecycle controls.

This is a scope contract only. It does not implement a dashboard, runtime
control endpoints, telemetry emitters, order-entry workflows, or deployment
packaging.

## MVP Goal

The first dashboard should answer one practical question:

```text
What is this local NTPRO node doing right now, and can the operator request a
small lifecycle action without touching trading internals?
```

The MVP is not a full trading frontend.

## Included Scope

| Area | MVP inclusion | Source contract |
| --- | --- | --- |
| System status viewing | Show node environment, lifecycle state, health, last transition, and redacted last error. | `docs/architecture/node_lifecycle_state_machine.md` and `docs/architecture/observability_state_model.md`. |
| Alert viewing | Show active alert counts and redacted alert summaries. | `docs/architecture/observability_state_model.md`. |
| Node lifecycle viewing | Show `stopped`, `starting`, `running`, `pausing`, `paused`, `resuming`, `stopping`, or `error` when available. | `docs/architecture/node_lifecycle_state_machine.md`. |
| Start control | Allow a future `start` request only when backend control support exists and state is valid. | `docs/architecture/control_api_contract.md`. |
| Stop control | Allow a future `stop` request only when backend control support exists and state is valid. | `docs/architecture/control_api_contract.md`. |
| Pause control | Show as contract-bound future control; disabled or unsupported until runtime support exists. | `docs/architecture/control_api_contract.md`. |
| Resume control | Show as contract-bound future control; disabled or unsupported until runtime support exists. | `docs/architecture/control_api_contract.md`. |

The MVP may show read-only summaries for data sources, execution gateways, risk,
and portfolio only through the observability model. It must not read mutable
engine/cache/message-bus internals directly.

## Excluded Scope

| Exclusion | Reason |
| --- | --- |
| Manual order entry | Order-entry controls are explicitly out of scope and would change trading risk. |
| Order modification or cancellation | Not part of lifecycle control and requires separate high-risk contract. |
| Strategy parameter hot reload | Mutates trading behavior and requires separate runtime contract. |
| Multi-user permissions | First dashboard is local/operator scope, not a shared multi-user product. |
| Complex asset management | Full asset/account management is outside dashboard MVP. |
| Full trading frontend | MVP is status/control boundary, not a professional trading terminal. |
| Docker delivery requirement | First implementation should not block on Docker packaging. |
| Raw adapter/venue payload inspection | Belongs to controlled fixtures/evidence, not dashboard MVP. |
| Credential editing or display | Secrets and credential material must never be dashboard state. |

## MVP Pages Or Panels

| Panel | Required content | Explicitly excluded |
| --- | --- | --- |
| Overview | Node lifecycle state, health, environment, last transition, last error summary. | Runtime internals, raw logs, raw stack traces. |
| Alerts | Alert counts by severity, active alert summaries, last changed timestamp. | Secrets, raw exception payloads, request/response bodies. |
| Data Sources | Source alias, provider/venue, connection status, freshness, last redacted error. | API keys, raw market data, private account payloads. |
| Execution Gateways | Gateway alias, venue, connection/started state, redacted order-count summary. | Order entry, raw orders, raw fills, raw venue reports. |
| Risk | Trading state, health, redacted rejection summary. | Full risk config, raw strategy/order commands. |
| Portfolio | Redacted account/position counts and optional local-only coarse PnL summary. | Raw account objects, credential data, shared/public monetary disclosure. |
| Controls | Contract-bound start/stop/pause/resume buttons or disabled placeholders. | Restart, reconnect, order controls, strategy hot reload. |

## Control Policy For MVP

MVP control UI must follow these rules:

- Controls are disabled when backend support is unavailable.
- Controls are disabled when the current lifecycle state is invalid for the
  requested action.
- Every control request must return the NARCH-005 response shape.
- Pause/resume must not be presented as implemented until runtime support is
  added by a later task.
- Stop must be clearly separated from order cancellation.
- No control may place, modify, cancel, synthesize, or replay orders.
- No control may mutate cache, message bus, adapter credentials, strategy
  parameters, or raw runtime internals.

## Data Policy

Dashboard MVP data must be local, read-only, and redacted:

| Data type | MVP policy |
| --- | --- |
| Secrets/credentials | Forbidden. |
| Account ids | Redacted aliases by default. |
| Monetary values | Disabled or local/operator-only coarse summaries until a later policy approves display. |
| Raw orders/fills | Forbidden. |
| Raw venue payloads | Forbidden. |
| Logs/errors | Redacted summaries only. |
| Adapter status | Fixture/mock/sandbox-safe status by default. |

## Acceptance Contract For Future Implementation

A future dashboard implementation is inside MVP scope only if it satisfies all
of these points:

- Uses the observability state model rather than direct mutable internals.
- Implements no manual order-entry workflow.
- Implements no strategy hot-reload workflow.
- Implements no multi-user permission system.
- Keeps controls limited to start, stop, pause, and resume contract actions.
- Shows unsupported controls as disabled or unsupported, not silently active.
- Runs local validation without requiring live exchange credentials.
- Provides evidence for UI scope, state rendering, redaction, and disabled
  unsupported controls.

## Follow-Up Tasks

| Need | Follow-up |
| --- | --- |
| Stable runtime status DTOs | Later observability implementation task. |
| Dashboard UI implementation | Later dashboard implementation task. |
| Control backend implementation | Later control-plane task using NARCH-005. |
| Redaction policy details | Later dashboard data policy task if needed. |
| Reconnect controls | Later task after start/stop/pause/resume are stable. |
| Release packaging | Later release/binary packaging task; Docker is not MVP-blocking. |
