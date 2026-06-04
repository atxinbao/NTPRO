# NTPRO Control API Contract

Date: 2026-06-04
Executor: Codex
Task ID: NARCH-005

## Purpose

This document defines the future control action contract for NTPRO. It is
intended for later local dashboard and operator-control work.

This is a contract document only. It does not implement control endpoints,
dashboard UI, order-entry controls, runtime code, adapter behavior, or live
trading behavior.

## Scope

The contract covers these future actions:

- `start`
- `stop`
- `restart`
- `pause_trading`
- `resume_trading`
- `reconnect_data`
- `reconnect_execution`

The contract uses the lifecycle states defined in
`docs/architecture/node_lifecycle_state_machine.md` and the read-only status
model defined in `docs/architecture/observability_state_model.md`.

## Contract Rules

- Control actions request state transitions; callers must not mutate lifecycle
  state directly.
- Control actions must be serialized per node. A later implementation should
  reject or queue overlapping actions explicitly.
- Every action response must include the accepted/rejected decision, current
  lifecycle state, and a redacted reason on failure.
- Every action must be auditable through local evidence: action id, requested
  action, previous state, resulting state, timestamp, and failure reason.
- Control actions must not expose or accept secrets, credentials, private keys,
  auth headers, signed payloads, or raw venue/account payloads.
- Control actions must not place, modify, cancel, or synthesize orders. Order
  entry and order management controls are explicitly out of scope.
- Reconnect actions must not imply production endpoint access during validation.
  Fixture, mock, dry-run, or sandbox evidence remains the default validation
  mode.

## Request Shape

| Field | Type | Meaning |
| --- | --- | --- |
| `action_id` | string | Stable id for dedupe and evidence. |
| `action` | enum | One of the actions in this contract. |
| `target` | object | Node, data source, or execution gateway target. |
| `requested_at` | timestamp | Request time. |
| `reason` | string | Short operator reason, redacted before display. |
| `mode` | enum | `validate_only` or `apply`. |
| `timeout_ms` | integer | Optional action timeout. |

`target` must use stable aliases from the future observability model. It must
not contain raw credentials, raw account objects, or adapter private config.

## Response Shape

| Field | Type | Meaning |
| --- | --- | --- |
| `action_id` | string | Matches the request. |
| `status` | enum | `accepted`, `rejected`, `running`, `succeeded`, `failed`, or `cancelled`. |
| `previous_state` | lifecycle state | State before action processing. |
| `current_state` | lifecycle state | State when the response is produced. |
| `started_at` | timestamp | Action start time when accepted. |
| `finished_at` | timestamp | Action finish time when complete. |
| `error_code` | string | Stable error code when rejected or failed. |
| `message` | string | Redacted human-readable result. |
| `observability_ref` | string | Optional reference to the resulting observability snapshot. |

## Action Contracts

### `start`

| Field | Contract |
| --- | --- |
| Purpose | Start a stopped node. |
| Allowed current states | `stopped`. |
| Rejected states | `starting`, `running`, `pausing`, `paused`, `resuming`, `stopping`, `error`. |
| Expected transition | `stopped -> starting -> running` on success. |
| Current runtime anchor | `LiveNode::start` and `NautilusKernel::start_async`. |
| Required evidence | Previous state, startup result, client connection summary, final state, failure reason if any. |
| Failure modes | Already running, invalid state, config error, startup timeout, data/exec client connection failure, startup abort, runtime fault. |

Current `LiveNode::start` rejects a running node as `Already running`. Future
control should reject every non-`stopped` state with a stable error code.

### `stop`

| Field | Contract |
| --- | --- |
| Purpose | Stop or clean up a node. |
| Allowed current states | `starting`, `running`, `pausing`, `paused`, `resuming`, `error`. |
| Idempotent state | `stopped`. |
| Rejected states | None by default; overlapping `stopping` should return already stopping. |
| Expected transition | active state -> `stopping -> stopped` on success. |
| Current runtime anchor | `LiveNode::stop`, `LiveNodeHandle::stop`, `NautilusKernel::stop_trader`, and client disconnect paths. |
| Required evidence | Previous state, stop reason, disconnect summary, cleanup status, final state. |
| Failure modes | Not running in current direct `LiveNode::stop`, disconnect failure, cleanup timeout, component fault, unknown state. |

Future operator control should treat stop from `stopped` as idempotent or report
already stopped without side effects. Current direct `LiveNode::stop` is stricter
and rejects when the node is not running.

### `restart`

| Field | Contract |
| --- | --- |
| Purpose | Stop and start the same node using an explicit two-phase action. |
| Allowed current states | `running`, `paused`, `error` after cleanup decision. |
| Rejected states | `starting`, `pausing`, `resuming`, `stopping`. |
| Expected transition | `running -> stopping -> stopped -> starting -> running` on success. |
| Current runtime anchor | No single restart API. Compose future stop and start actions. |
| Required evidence | Stop evidence, start evidence, final state, whether config changed. |
| Failure modes | Stop failed, start failed, config invalid, reconnect failed, action timeout. |

Restart must not skip the `stopping` and `stopped` states. A failed stop must
not continue into start.

### `pause_trading`

| Field | Contract |
| --- | --- |
| Purpose | Hold future trading actions while preserving read-only observability. |
| Allowed current states | `running`. |
| Rejected states | `stopped`, `starting`, `pausing`, `paused`, `resuming`, `stopping`, `error`. |
| Expected transition | `running -> pausing -> paused` on success. |
| Current runtime anchor | Future contract only. Component-level `Resuming` exists, but top-level pause is not implemented. |
| Required evidence | Previous state, accepted/held command boundary, final state, risk/trading gate summary. |
| Failure modes | Unsupported action, invalid state, outstanding transition, unable to hold new trading actions. |

This action must not cancel existing orders unless a later explicitly approved
order-control task defines that behavior. It must not be advertised as currently
implemented.

### `resume_trading`

| Field | Contract |
| --- | --- |
| Purpose | Resume trading after a completed pause. |
| Allowed current states | `paused`. |
| Rejected states | `stopped`, `starting`, `running`, `pausing`, `resuming`, `stopping`, `error`. |
| Expected transition | `paused -> resuming -> running` on success. |
| Current runtime anchor | Future contract only. |
| Required evidence | Previous state, resumed gate summary, final state. |
| Failure modes | Unsupported action, invalid state, stale pause state, resume failure. |

Resume must not submit queued orders automatically unless a later order-control
contract explicitly approves that behavior.

### `reconnect_data`

| Field | Contract |
| --- | --- |
| Purpose | Reconnect one configured data source or all data sources. |
| Allowed current states | `running`, `paused`. |
| Rejected states | `stopped`, `starting`, `stopping`, `error` unless cleanup/recovery explicitly approves it. |
| Expected transition | Node remains `running` or `paused`; data source connection moves through reconnect status. |
| Current runtime anchor | Data client `connect`/`disconnect` paths and `NautilusKernel::connect_data_clients`. |
| Required evidence | Target source alias, previous connection state, disconnect/connect result, subscription restoration summary. |
| Failure modes | Source not configured, unsupported adapter, disconnect failure, connect failure, subscription restore failure, stale config. |

Reconnect data must not expose credentials and must not require live endpoint
access in routine validation.

### `reconnect_execution`

| Field | Contract |
| --- | --- |
| Purpose | Reconnect one configured execution gateway or all execution gateways. |
| Allowed current states | `running`, `paused`. |
| Rejected states | `stopped`, `starting`, `stopping`, `error` unless cleanup/recovery explicitly approves it. |
| Expected transition | Node remains `running` or `paused`; execution gateway connection moves through reconnect status. |
| Current runtime anchor | Execution client `connect`/`disconnect` paths and `NautilusKernel::connect_exec_clients`. |
| Required evidence | Target gateway alias, previous connection state, disconnect/connect result, reconciliation summary. |
| Failure modes | Gateway not configured, unsupported adapter, disconnect failure, connect failure, reconciliation failure, stale config. |

Reconnect execution must not place, modify, or cancel orders. Reconciliation
evidence should summarize counts and results, not raw venue/account payloads.

## Error Codes

| Code | Meaning |
| --- | --- |
| `invalid_state` | Action is not allowed from the current lifecycle state. |
| `already_in_progress` | Another control action is running for the target node. |
| `already_stopped` | Stop was requested for an already stopped node. |
| `already_running` | Start was requested for an already running node. |
| `unsupported_action` | Contract exists, but implementation is not available. |
| `target_not_found` | Target alias does not resolve to a configured node/source/gateway. |
| `target_not_configured` | Target exists conceptually but is not configured in this run. |
| `timeout` | Action did not complete within the requested timeout. |
| `runtime_error` | Runtime returned an explicit failure. |
| `redacted` | Details exist but cannot be displayed in dashboard state. |

## Current Implementation Status

| Action | Current implementation status |
| --- | --- |
| `start` | Runtime has `LiveNode::start`, but no dashboard/control API wrapper. |
| `stop` | Runtime has `LiveNode::stop` and `LiveNodeHandle::stop`, but no dashboard/control API wrapper. |
| `restart` | No single runtime control action. Future composition of stop then start. |
| `pause_trading` | Future contract only. |
| `resume_trading` | Future contract only. |
| `reconnect_data` | Data clients and kernel have connect/disconnect paths, but no stable external reconnect action. |
| `reconnect_execution` | Execution clients and kernel have connect/disconnect paths, but no stable external reconnect action. |

## Forbidden Controls

The following remain out of scope for this contract:

- order entry;
- order modification;
- order cancellation;
- strategy parameter mutation;
- direct cache mutation;
- direct message-bus command injection;
- direct adapter credential changes;
- arbitrary raw venue requests;
- production live endpoint validation without explicit later approval.

## Follow-Up Boundaries

- A later implementation task must add stable request/response types before any
  UI consumes this contract.
- NDASH-001 should choose which actions, if any, belong in the dashboard MVP.
- Runtime control implementation must use the observability model for feedback
  instead of reading or mutating engine internals from UI code.
- Any implementation that changes live trading behavior, order flow, adapter
  behavior, or persistence semantics must be treated as high risk and stop for
  review.
