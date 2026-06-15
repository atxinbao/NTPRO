# NTPRO v0.7.0 Read-Only Binance Testnet Boundary

Date: 2026-06-14
Executor: Codex
Milestone: v0.7.0 real testnet read-only connectivity proof
Risk: high

## Plain Chinese Summary

v0.7.0 只能跨过一个边界：从 v0.6.1 的离线 dry-run，进入真实 Binance testnet 的
**只读连通性证明**。

大白话说：v0.7.0 可以在人工打开网络 gate 后，访问 Binance testnet 的公开/只读接口，
证明 HTTP 连得上、返回结构能被记录成 artifact。它不能下单，不能做真实账户交易，不能
使用真实资金，不能写成生产交易可用。

默认 CI 仍然不联网。真实联网只能在 manual online gate 中运行，并且必须 fail-closed。
Dashboard 只能读取已经生成的 artifact，不能发起 probe，不能读取 secret，不能提供连接或
下单按钮。

## Scope Claim

```text
real Binance testnet read-only connectivity proof
```

v0.7.0 includes:

- real Binance testnet HTTP read-only connectivity proof;
- artifact schema for HTTP read-only probe results;
- optional/manual WebSocket read-only probe schema;
- fail-closed network opt-in guard;
- env-only credential policy for optional online mode;
- Dashboard read-only display of generated probe artifacts;
- dual verification:
  - default offline gate;
  - manual online gate.

v0.7.0 does not include:

- real order submission;
- simulated order submission presented as real exchange behavior;
- live trading;
- production trading;
- production Binance connectivity;
- real funds;
- account balance mutation;
- order placement, cancel, replace, or amend;
- Dashboard network initiation;
- Dashboard secret access;
- storing API key or API secret values in artifacts, stdout, logs, docs, PR body,
  or screenshots.

## Fail-Closed Network Opt-In Contract

Real testnet network access is forbidden unless every condition below is true:

```text
--allow-testnet-network
NTPRO_ALLOW_TESTNET_NETWORK=1
config environment = testnet
order_submission = disabled
real_orders_submitted = false
```

If any condition is missing or false, the workflow must stay offline and write
an offline/blocked artifact instead of opening a socket.

The guard must be evaluated before any HTTP client or WebSocket client is
created. The error mode must be explicit and owner-visible:

```text
network_gate_status = blocked
network_attempted = false
testnet_connection = false
```

## Connection Field Semantics

v0.7 distinguishes production venue contact from public testnet read-only
network contact. The release artifacts must not use one ambiguous field for
both meanings.

Workflow `summary.json`, `boundary.json`, and `manifest.summary` must include:

| Field | Meaning |
| --- | --- |
| `production_venue_connection` | True only for production venue connectivity. Must remain false in v0.7. |
| `testnet_public_network_connection` | True only after the Binance testnet public read-only HTTP response succeeds and its shape is validated. |
| `external_network_attempted` | True only after the workflow actually attempts an external network request. |
| `external_venue_connection` | Compatibility field retained for older local artifacts. It must remain false for v0.7 read-only testnet proof. |
| `testnet_connection` | Compatibility/summary field for v0.7 testnet read-only proof. |
| `network_attempted` | Compatibility/summary field for v0.7 network attempt proof. |

Dashboard should show `production_venue_connection`,
`testnet_public_network_connection`, and `external_network_attempted` as the
primary labels. It may keep reading legacy fields only as fallback.

## Gate Separation

### Default Offline Gate

The default gate runs in local development and normal CI. It must not require
Binance credentials and must not open sockets.

Script:

```bash
scripts/ai/verify_v07_default_offline_gate.sh
```

Required proof:

- CLI help exposes the opt-in contract;
- offline config validation passes;
- offline artifact schema validation passes;
- `network_attempted=false`;
- `testnet_connection=false`;
- no secret value is present in artifacts or logs.
- both dry-run and blocked connectivity-probe artifacts remain offline when
  opt-in is missing.

### Manual Online Gate

The manual online gate is the only place where real Binance testnet read-only
network access may occur.

Default classified preflight:

```bash
scripts/ai/verify_v07_manual_online_gate.sh
```

The default preflight intentionally unsets `NTPRO_ALLOW_TESTNET_NETWORK`, passes
`--allow-testnet-network`, and proves the missing environment opt-in keeps
`network_attempted=false`.

Real read-only online proof:

```bash
NTPRO_V07_MANUAL_ONLINE=1 \
NTPRO_ALLOW_TESTNET_NETWORK=1 \
scripts/ai/verify_v07_manual_online_gate.sh
```

The real online proof opens only the configured Binance testnet public HTTP
time endpoint. Connectivity proof requires `network_attempted=true`,
`testnet_connection=true`, `error_code=none`, and
`response_shape_validated=true` for the Binance server-time response. A stable
classified `error_code` is diagnostic/classification evidence only; it is not
connectivity proof. It does not submit, cancel, replace, or amend orders.

Required proof:

- all fail-closed conditions are true;
- no order path is enabled;
- no real order is submitted;
- no secret value is written to artifacts or logs;
- HTTP probe artifact records success/failure without leaking secrets;
- manual operator explicitly accepted that this is testnet read-only.

Manual online gate failures must not be hidden as default CI failures. They
must be recorded as online-gate evidence and may block v0.7.0 release only when
the release decision explicitly requires online proof.

## HTTP Probe Artifact Schema

HTTP probe is the v0.7.0 primary path.

Suggested artifact path:

```text
testnet/http_connectivity_probe.json
```

Schema version:

```text
ntpro.v07_binance_testnet_http_probe.v1
```

Required fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `schema_version` | string | yes | Must equal `ntpro.v07_binance_testnet_http_probe.v1`. |
| `run_id` | string | yes | Effective workflow run id. |
| `environment` | string | yes | Must be `testnet` for online attempts. |
| `product` | string | yes | Example: `spot`. |
| `endpoint_kind` | string | yes | `http_read_only`. |
| `endpoint_url_redacted` | string | yes | May include host/path, never credentials. |
| `network_gate_status` | string | yes | `blocked`, `allowed`, `attempted`, `completed`, or `failed`. |
| `network_attempted` | bool | yes | True only after socket creation is allowed and attempted. |
| `testnet_connection` | bool | yes | True only when the read-only HTTP response is successful and validated. |
| `order_submission` | string | yes | Must be `disabled`. |
| `real_orders_submitted` | bool | yes | Must be false. |
| `credential_policy` | string | yes | Must describe env-only/no-persistence policy. |
| `api_key_present` | bool | yes | Presence only, never value. |
| `api_secret_present` | bool | yes | Presence only, never value. |
| `request_method` | string | yes | Must be a read-only method such as `GET`. |
| `request_target` | string | yes | Redacted path or endpoint label. |
| `response_status_code` | number/null | yes | Null when blocked before network. |
| `response_shape` | string | yes | Schema/shape label, not raw response if it may include sensitive data. |
| `response_shape_validated` | bool | yes | True only when `/api/v3/time` returned a JSON object with numeric `serverTime`. |
| `latency_ms` | number/null | yes | Optional when blocked or failed before response. |
| `diagnostic` | string | yes | Human-readable result without secrets. |
| `generated_at` | string | yes | Timestamp. |

Forbidden fields:

- `api_key`;
- `api_secret`;
- `signature`;
- `listen_key`;
- raw authorization headers;
- raw request body containing secrets;
- raw response body if it can contain account or credential material.

## Optional WebSocket Probe Artifact Schema

WebSocket proof is optional/manual and must not be a default CI release blocker.

Suggested artifact path:

```text
testnet/ws_connectivity_probe.json
```

Schema version:

```text
ntpro.v07_binance_testnet_ws_probe.v1
```

Required fields:

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `schema_version` | string | yes | Must equal `ntpro.v07_binance_testnet_ws_probe.v1`. |
| `run_id` | string | yes | Effective workflow run id. |
| `endpoint_kind` | string | yes | `websocket_read_only`. |
| `network_gate_status` | string | yes | Same enum as HTTP probe. |
| `network_attempted` | bool | yes | True only when socket creation was attempted. |
| `testnet_connection` | bool | yes | True only after connection/open handshake proof. |
| `subscription_attempted` | bool | yes | Must remain false unless a later task explicitly approves a public read-only subscription. |
| `message_count` | number | yes | Must be zero for handshake-only proof. |
| `order_submission` | string | yes | Must be `disabled`. |
| `real_orders_submitted` | bool | yes | Must be false. |
| `diagnostic` | string | yes | No secrets. |
| `generated_at` | string | yes | Timestamp. |

## Credential and Secret Policy

Secrets are allowed only as environment variables in the running process when
manual online gate explicitly requires them.

Allowed artifact values:

- env var names;
- presence booleans;
- redaction status;
- policy labels;
- non-secret endpoint labels.

Forbidden artifact, stdout, log, docs, PR body, and screenshot values:

- API key value;
- API secret value;
- signature value;
- listen key;
- account identifiers if not required for read-only connectivity proof;
- any raw credential-bearing header.

The implementation must treat secret leakage as a hard FAIL, not a warning.

## Dashboard Boundary

Dashboard may display v0.7 artifacts only after they already exist on disk.

Allowed Dashboard behavior:

- read local artifact files;
- display `network_gate_status`;
- display `network_attempted`;
- display `testnet_connection`;
- display `order_submission=disabled`;
- display `real_orders_submitted=false`;
- display credential policy labels and presence booleans.

Forbidden Dashboard behavior:

- start HTTP probe;
- start WebSocket probe;
- read environment secret values;
- display secret values;
- provide connect, order, cancel, amend, or trade buttons;
- turn manual online gate into a background Dashboard action.

## Threat Model

| Threat | Impact | Required Mitigation |
| --- | --- | --- |
| Accidental real network from default CI | CI becomes non-deterministic and may leak environment assumptions. | Default gate must not open sockets; online proof must require manual gate and full opt-in. |
| Accidental order submission | Testnet or production state mutation. | `order_submission=disabled`, no order endpoints, `real_orders_submitted=false`, and tests/gates that fail if any order path is enabled. |
| Secret leakage in artifacts/logs | Credential exposure. | Env-only secrets, presence booleans only, redaction checks, no raw headers or bodies with secrets. |
| Production endpoint confusion | Real production exchange contact. | Config environment must be `testnet`; endpoint host must be testnet; production hosts must hard fail. |
| Dashboard privilege escalation | UI becomes a network actuator. | Dashboard read-only artifact model; no probe or credential actions. |
| WebSocket scope creep | Optional probe becomes release blocker or subscription engine. | WebSocket remains optional/manual and handshake/read-only only unless a later task changes scope. |

## Required Future Task Mapping

- `V070-001`: implement fail-closed network opt-in guard.
- `V070-002`: harden env-only credential policy.
- `V070-003`: add HTTP read-only probe artifacts.
- `V070-004`: add optional/manual WebSocket probe artifacts.
- `V070-005`: display v0.7 probe artifacts in Dashboard read-only mode.
- `V070-006`: add dual-gate verification for offline default and manual online.
- `V070-007`: prepare v0.7.0 readiness report and release notes.

## Release Decision Rule

v0.7.0 can be considered ready only if:

- offline default gate passes without network;
- manual online gate passes when explicitly run with real testnet opt-in;
- artifacts prove read-only behavior;
- no secrets are leaked;
- no order path is enabled;
- Dashboard remains artifact-only;
- release notes state `includes` and `does not include` without production
  trading ambiguity.
