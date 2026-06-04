# NTPRO Observability State Model

Date: 2026-06-04
Executor: Codex
Task ID: NARCH-004

## Purpose

This document defines the future dashboard-readable observability state model
for NTPRO. It gives later dashboard and telemetry work a stable read-only target
without exposing mutable engine internals.

This is a model document only. It does not implement telemetry emitters,
dashboard UI, control endpoints, adapter changes, or runtime behavior changes.

## Model Rules

- Observability is read-only.
- Observability state must be derived from stable runtime contracts, not direct
  mutable access to engines, cache, message bus, adapters, or account objects.
- Every field must have a freshness timestamp or be explicitly documented as a
  static/config field.
- Missing data must be represented as `unknown`, `not_configured`,
  `not_supported`, or `stale`, not silently reported as healthy.
- Secrets, credentials, tokens, API keys, private keys, raw auth headers,
  password-like values, and raw signed payloads are forbidden.
- Raw order payloads, raw venue payloads, and raw account objects are not
  dashboard state. They belong in controlled forensic evidence or fixtures.
- Account identifiers and monetary values are sensitive operational data. The
  default dashboard model should expose redacted aliases and coarse summaries
  unless an explicitly local/operator-only display mode is approved later.

## Top-Level Snapshot

Future dashboard state should be published or queried as a single read-only
snapshot:

| Field | Type | Meaning |
| --- | --- | --- |
| `schema_version` | string | Observability schema version. |
| `generated_at` | timestamp | Time the snapshot was assembled. |
| `source` | enum | `live`, `sandbox`, `backtest`, `fixture`, or `unknown`. |
| `system` | `SystemStatus` | Node and runtime status. |
| `data_sources` | list of `DataSourceStatus` | Data adapter/provider summaries. |
| `execution_gateways` | list of `ExecutionGatewayStatus` | Execution adapter/gateway summaries. |
| `risk` | `RiskStatus` | Trading gate and rejection summary. |
| `portfolio` | `PortfolioSummary` | Redacted account/position/PnL summary. |
| `alerts` | `AlertSummary` | Current warning/error summary. |
| `gaps` | list of `ObservabilityGap` | Known unavailable or stale fields. |

The snapshot is a contract target. Current runtime modules do not yet expose a
single struct or endpoint with this shape.

## Status Enums

### Health

| Value | Meaning |
| --- | --- |
| `healthy` | Fresh data and no known error. |
| `degraded` | Runtime is usable but one or more components are stale, disconnected, delayed, or partially unavailable. |
| `error` | A component failed or the node lifecycle is in `error`. |
| `stale` | Last update is older than the configured freshness threshold. |
| `unknown` | The field cannot be determined from current sources. |

### Connection

| Value | Meaning |
| --- | --- |
| `connected` | Client/provider/gateway is connected. |
| `connecting` | Connection is in progress. |
| `disconnected` | Client/provider/gateway is not connected. |
| `disconnecting` | Disconnect is in progress. |
| `not_configured` | Source is absent for this runtime. |
| `unknown` | Current runtime source does not expose the state. |

### Severity

| Value | Meaning |
| --- | --- |
| `info` | Useful operator context. |
| `warning` | Needs attention but does not block operation. |
| `error` | Current or recent failure. |
| `critical` | Trading/runtime should not continue without operator review. |

## SystemStatus

| Field | Meaning | Current source or gap |
| --- | --- | --- |
| `environment` | Backtest, sandbox, live, or unknown. | Runtime config and node setup. |
| `trader_id` | Trader identifier. | `crates/system` trader/kernel context. |
| `node_state` | One of the NARCH-003 lifecycle states. | Map from `crates/live/src/node.rs` `NodeState` plus future lifecycle contract. |
| `health` | Overall system health. | Derived, not currently a stable runtime DTO. |
| `started_at` | Start timestamp. | Trader keeps timestamps such as `ts_started`; stable summary missing. |
| `stopped_at` | Stop timestamp. | Trader keeps timestamps such as `ts_stopped`; stable summary missing. |
| `last_transition_at` | Last lifecycle transition time. | Future lifecycle status object required. |
| `component_counts` | Counts by component state. | Component states exist, but aggregation contract is missing. |
| `last_error` | Redacted lifecycle/runtime error summary. | Future status/error collection required. |

System status must not expose actor registries, strategy internals, mutable
kernel fields, or message-bus routing tables.

## DataSourceStatus

| Field | Meaning | Current source or gap |
| --- | --- | --- |
| `source_id` | Stable source alias. | Future DTO required. |
| `source_kind` | `adapter`, `catalog`, `sandbox`, `fixture`, or `unknown`. | Adapter/config classification. |
| `venue_or_provider` | Public venue/provider name. | Adapter crate/config. |
| `classification` | Supported, sandbox-only, fixture-only, deferred, or removed. | `docs/integrations/adapter_support_matrix.md`. |
| `connection` | Connection enum. | Some clients expose `is_connected`; no unified data-client status model. |
| `subscription_counts` | Counts by data type, not raw subscriptions. | DataEngine tracks subscriptions internally; stable summary missing. |
| `last_event_at` | Last accepted data event timestamp. | Data processing path exists; stable summary missing. |
| `lag_ms` | Freshness lag when measurable. | Future telemetry field. |
| `health` | Data source health. | Derived from connection, freshness, and errors. |
| `last_error` | Redacted parser/connection/request error. | Future error summary field. |

Data source status must not expose API keys, auth headers, signed URLs,
credential file paths, raw market-data payloads, or venue private account
payloads.

## ExecutionGatewayStatus

| Field | Meaning | Current source or gap |
| --- | --- | --- |
| `gateway_id` | Stable redacted gateway alias. | Future DTO required. |
| `venue` | Public venue name. | Execution client core has venue. |
| `connection` | Connection enum. | `ExecutionClientCore` exposes `is_connected`. |
| `started` | Whether the gateway has started. | `ExecutionClientCore` exposes `is_started`. |
| `account_ref` | Redacted account alias. | Execution client core has account id; dashboard should not expose raw id by default. |
| `orders_open` | Open order count. | Cache has order count query paths. |
| `orders_inflight` | In-flight order count. | Cache has order count query paths. |
| `orders_closed` | Closed order count. | Cache has order count query paths. |
| `last_report_at` | Last execution report timestamp. | Future summary required. |
| `last_reconciliation_at` | Last reconciliation timestamp. | Reconciliation paths exist; stable summary missing. |
| `last_error` | Redacted gateway/order-routing error. | Future error summary field. |

Execution gateway status must not expose raw orders, raw fills, raw venue order
payloads, full account objects, credentials, or signing material.

## RiskStatus

| Field | Meaning | Current source or gap |
| --- | --- | --- |
| `trading_state` | Active, reducing, halted, or unknown. | `RiskEngine::trading_state`. |
| `health` | Risk subsystem health. | Derived. |
| `command_count` | Commands observed by risk. | Risk engine stores counters, but stable getter/DTO is missing. |
| `event_count` | Events observed by risk. | Risk engine stores counters, but stable getter/DTO is missing. |
| `rejections_total` | Count of denied/rejected actions. | Future counter required. |
| `last_rejection` | Redacted reason and timestamp. | Order denial/rejection events exist; summary missing. |
| `limits_summary` | Coarse configured limits summary. | Config exists; values may be sensitive and should be redacted by default. |

Risk status may show whether trading is active, reducing, or halted. It should
not expose full risk configuration, secret strategy parameters, or raw order
commands by default.

## PortfolioSummary

| Field | Meaning | Current source or gap |
| --- | --- | --- |
| `display_mode` | `redacted`, `operator_local`, or `disabled`. | Future dashboard policy. |
| `account_count` | Number of known accounts. | Cache/account manager paths. |
| `account_refs` | Redacted aliases, not raw account ids by default. | Future redaction layer required. |
| `open_positions` | Count of open positions. | Cache and portfolio state paths. |
| `net_exposure_by_currency` | Coarse exposure summary. | `Portfolio::net_exposures` exists. |
| `realized_pnl_by_currency` | Optional local/operator-only summary. | `Portfolio::realized_pnls` exists. |
| `unrealized_pnl_by_currency` | Optional local/operator-only summary. | `Portfolio::unrealized_pnls` exists. |
| `last_snapshot_at` | Latest portfolio snapshot timestamp. | `Portfolio::snapshots` exists; stable summary missing. |
| `missing_price_count` | Number of instruments missing pricing data. | Portfolio tracks missing-price state internally; stable summary missing. |

Portfolio state is sensitive. Public or shared dashboards should default to
counts and redacted aliases. Monetary values require an explicit later policy
for local/operator-only display.

## AlertSummary

| Field | Meaning | Current source or gap |
| --- | --- | --- |
| `counts_by_severity` | Number of active alerts by severity. | Future alert aggregator required. |
| `active` | List of current alert summaries. | Future alert aggregator required. |
| `last_changed_at` | Last alert update timestamp. | Future alert aggregator required. |

Each active alert should include:

| Field | Meaning |
| --- | --- |
| `alert_id` | Stable alert id. |
| `severity` | `info`, `warning`, `error`, or `critical`. |
| `source` | System, data, execution, risk, portfolio, adapter, persistence, or verification. |
| `message` | Short redacted human-readable message. |
| `first_seen_at` | First observed timestamp. |
| `last_seen_at` | Last observed timestamp. |
| `status` | `active`, `acknowledged`, or `cleared`. |

Alert messages must be redacted before display. Raw exception payloads, headers,
tokens, credentials, request bodies, and raw venue data must not be copied into
alert state.

## ObservabilityGap

When a field is missing or stale, the snapshot should carry an explicit gap:

| Field | Meaning |
| --- | --- |
| `field_path` | Path such as `data_sources[0].last_event_at`. |
| `reason` | `not_implemented`, `not_configured`, `not_supported`, `stale`, or `redacted`. |
| `owner_task` | Follow-up task when known. |
| `notes` | Short explanation without secrets. |

## Current Source Matrix

| Area | Current usable source | Gap |
| --- | --- | --- |
| Node lifecycle | `LiveNodeHandle::state`, `NodeState`, NARCH-003 contract. | No stable top-level status DTO with last transition/error metadata. |
| System/trader | Trader component lifecycle and timestamps. | No read-only component-count summary. |
| Data source | DataEngine subscription/cache paths and adapter config. | No unified data client connection/freshness DTO. |
| Execution gateway | `ExecutionClientCore` connected/started flags and cache order counts. | No redacted gateway summary DTO. |
| Risk | `RiskEngine::trading_state`, command/event counters, denial events. | No rejection counter/last rejection summary DTO. |
| Portfolio | Portfolio PnL/exposure/snapshot methods and cache position/account views. | No redacted portfolio dashboard summary DTO. |
| Alerts | Logs and runtime errors. | No alert aggregator or severity policy. |

## Follow-Up Boundaries

- NARCH-005 should define which future control actions may consume this state.
- NDASH-001 should decide the dashboard MVP fields and display policy.
- Runtime implementation tasks must add stable DTOs before dashboard code reads
  engine internals.
- Adapter tasks must provide status summaries through fixture/mock/sandbox
  evidence, not live credentials.
- Release tasks should keep observability checks local unless a later release
  policy explicitly promotes them.
