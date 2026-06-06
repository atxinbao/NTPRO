# NTPRO Local Multi-Node Runtime Contract

Date: 2026-06-06
Executor: Codex
Task ID: V02-002

## Purpose

This document defines the v0.2 product contract for the local multi-node
runtime foundation. It covers `ntpro-node`, `ntpro-supervisor`, `node_id`,
local process expectations, config ownership, lifecycle command boundaries, and
artifact layout.

This is a contract document only. It does not implement new CLI commands,
runtime code, Dashboard UI, control API endpoints, production exchange
connectivity, manual order entry, or release artifacts.

## Product Terms

| Term | Contract |
| --- | --- |
| `ntpro-node` | A Rust local trading runtime target. One node represents one independent runtime instance with one config, one lifecycle state, and one artifact directory. |
| `ntpro-supervisor` | A local manager for node registry, start, stop, status, logs, and metrics paths. It owns orchestration metadata, not trading semantics. |
| `node_id` | Stable local identifier used by supervisor commands and artifact paths. It must be unique within one supervisor registry. |
| node config | TOML config owned by the node. The supervisor stores the config path and may validate it, but it must not rewrite the config while a node is running. |
| node artifact root | Local filesystem directory containing status, pid/process metadata, logs, metrics, and run artifacts for one node. |

## v0.2 Runtime Boundary

v0.2 is scoped to local sandbox-first evidence:

- one supervisor manages local nodes on the same machine;
- one `node_id` maps to one configured local runtime target;
- local sandbox nodes may use Rust `LiveNode` sandbox start/stop evidence;
- local artifacts are the source of truth for status, logs, and minimal metrics;
- CLI control may call the local supervisor path in later V02 tasks.

v0.2 does not claim:

- production real-exchange live trading;
- remote or distributed supervisor operation;
- Dashboard UI;
- control API endpoint implementation;
- manual order entry;
- strategy parameter hot reload;
- order modification or cancellation controls;
- release tag or GitHub Release readiness.

## Process Model

Product architecture treats each node as an independent process target:

```text
ntpro-supervisor
  -> node_id: sandbox-a -> ntpro-node --config configs/sandbox-a.toml
  -> node_id: sandbox-b -> ntpro-node --config configs/sandbox-b.toml
```

Early tests may use a local harness or short-lived command path, but the
contract remains one node = one independent runtime target. Any implementation
that collapses multiple nodes into one mutable runtime must be explicitly
recorded as test-only and must not be described as product multi-node support.

The supervisor may track OS process metadata when a node is spawned as a child
process. If a later task uses an in-process harness for tests, the status
artifact must still show that process mode as `test_harness`, not as a full
spawned node process.

## Node ID Contract

`node_id` is the primary local key.

Allowed shape:

```text
[a-z0-9][a-z0-9._-]{0,63}
```

Rules:

- unique within one supervisor registry;
- stable across start/stop/status commands for the same configured node;
- not derived from display name alone;
- not used to imply account, venue, or credential identity;
- unsuitable IDs must be rejected before artifact paths are created.

Examples:

```text
sandbox-a
sandbox-b
live-smoke-01
```

## Config Ownership

Each node has exactly one config path in the supervisor registry.

The node config is responsible for:

- environment selection, such as sandbox-only in the first V02 runtime path;
- trader and instance identifiers;
- adapter or sandbox fixture selection;
- output preferences;
- shutdown behavior;
- runtime-specific validation inputs.

The supervisor is responsible for:

- storing the config path;
- checking whether the path exists before start;
- passing the path to `ntpro-node`;
- recording validation results in status artifacts;
- refusing to start a node if another active node owns the same `node_id`.

The supervisor must not mutate strategy parameters, adapter credentials, order
flow, risk settings, or venue/account state. Config changes require stopping or
recreating the node unless a later task explicitly defines a safe reload path.

## Local File Layout

Default local layout:

```text
runs/local-nodes/
  registry.json
  nodes/
    <node_id>/
      config.toml          # copied or referenced config snapshot for evidence
      pid.json             # process metadata when spawned
      status.json          # latest node status snapshot
      metrics.json         # minimal local counters and timestamps
      logs/
        stdout.log
        stderr.log
        events.log
      artifacts/
        summary.txt
        run.json
```

Path rules:

- `registry.json` records node ids, config paths, artifact roots, process mode,
  and last known status.
- `status.json` is the lightweight status source for `status` commands.
- `metrics.json` is optional until V02-007, but the path is reserved here.
- `logs/` stores local process output and lifecycle events only.
- `artifacts/` stores run-specific summaries produced by `ntpro-node`.

The layout is repository-local by default for development evidence. A later
release task may choose a user-level runtime directory, but V02 must not require
Docker or a system service manager.

## Lifecycle Command Boundary

The local supervisor contract covers these commands:

| Command | Contract | Current V02 status |
| --- | --- | --- |
| `register` | Add or update a stopped node entry with `node_id`, config path, and artifact root. | V02-005 target |
| `start` | Start a registered stopped node through the local `ntpro-node` path. | V02-006 target |
| `stop` | Stop a starting/running node and update status artifacts. | V02-006 target |
| `status` | Read registry and node status artifacts without mutating runtime state. | V02-006 target |
| `logs` | Read or locate local log artifacts. | V02-007 target |
| `metrics` | Read minimal local metrics JSON. | V02-007 target |

Invalid action rules must follow
`docs/architecture/node_lifecycle_state_machine.md` and
`docs/architecture/control_api_contract.md`.

## Status Artifact Contract

V02-002 reserves the following minimum status fields for later V02
implementation tasks:

| Field | Meaning |
| --- | --- |
| `schema_version` | Status artifact schema version. |
| `node_id` | Stable local node id. |
| `process_mode` | `spawned_process`, `test_harness`, or `unknown`. |
| `config_path` | Local path supplied to the node. |
| `artifact_root` | Local node artifact directory. |
| `lifecycle_state` | Contract state from `node_lifecycle_state_machine.md`. |
| `started_at` | Start timestamp when available. |
| `stopped_at` | Stop timestamp when available. |
| `last_transition_at` | Latest lifecycle transition timestamp. |
| `last_error` | Redacted human-readable failure summary when available. |
| `external_venue_connection` | Must be `false` for V02 sandbox-only evidence. |
| `real_orders_submitted` | Must be `false` for V02 sandbox-only evidence. |

V02-003 owns the Rust DTO implementation details. This document only defines
the product contract those DTOs must satisfy.

## Sandbox-Only First Release Behavior

The first V02 runtime path must use sandbox/mock/fixture behavior by default.

Allowed first-release behavior:

- start and stop sandbox-only `LiveNode` paths;
- use simulated execution client behavior already scoped by DRG-005;
- write local status, logs, metrics, and summary artifacts;
- prove two local sandbox nodes can be managed without external venue access.

Not allowed as V02-002 contract claims:

- real venue connectivity;
- real order submission;
- real account reconciliation;
- production adapter start/stop behavior;
- manual operator trading actions.

## Relationship To Existing Contracts

- `docs/architecture/node_lifecycle_state_machine.md` defines lifecycle states.
- `docs/architecture/control_api_contract.md` defines future action semantics.
- `docs/architecture/observability_state_model.md` defines future read-only
  status shape.
- `docs/rust-cutover/product/CLI_CAPABILITY_MATRIX.md` defines current CLI
  capability honesty.
- `docs/rust-cutover/scope/v0_2_local_multi_node_runtime.md` defines the active
  V02 scope.

## Completion Rule

V02 implementation tasks must not claim local multi-node runtime completion
until:

- V02-003 provides status DTOs;
- V02-004 provides sandbox-only real `ntpro-node` start/stop evidence;
- V02-005 provides supervisor registry evidence;
- V02-006 provides start/stop/status evidence;
- V02-007 provides log and metrics artifact evidence;
- V02-008 wires CLI controls through the supervisor path;
- V02-009 proves a two-node local sandbox smoke;
- V02-010 records the final PASS/FAIL readiness report.
