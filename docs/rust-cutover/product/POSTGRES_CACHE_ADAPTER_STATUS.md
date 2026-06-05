# PostgreSQL Cache Adapter Status

Date: 2026-06-05
Executor: Codex
Task ID: NAUDIT-005

## Classification

Status: `unsupported` for the NTPRO v0.2 product surface.

This classification applies to the PostgreSQL `CacheDatabaseAdapter` path in
`crates/infrastructure/src/sql/cache.rs`. It does not remove or downgrade the
existing `nautilus database init/drop` operational CLI commands.

## Boundary

Supported in v0.2:

- Rust in-memory cache paths covered by common/runtime tests.
- `nautilus database init` and `nautilus database drop` as existing PostgreSQL
  administration utilities.

Unsupported in v0.2:

- Treating PostgreSQL as a stable durable cache persistence backend.
- Release claims that order, position, account, snapshot, index, heartbeat, or
  synthetic instrument cache state can be fully persisted and restored through
  the PostgreSQL cache adapter.
- Using ignored PostgreSQL cache integration tests as release evidence.

## Rationale

The adapter still contains explicit unsupported operations, including:

- synthetic instrument load/add;
- actor and strategy load/delete;
- order and position delete;
- position load/add/update;
- order book add;
- funding rate add/load;
- venue-order and order-position indexes;
- order and position state snapshots;
- heartbeat persistence.

The PostgreSQL cache integration tests still include schema/FK blockers:

- `test_order_cancel_rejected_insert_and_load`;
- `test_order_modify_rejected_insert_and_load`.

Both are ignored with the reason:

```text
Waiting on PostgreSQL schema completion - needs FK constraints
```

## Promotion Gates

Before this adapter can move from `unsupported` to `experimental`, a later task
must at minimum:

- inventory every `not implemented for PostgreSQL cache adapter` operation;
- decide which operations are in scope for the first supported slice;
- add or migrate the required schema and FK constraints;
- restore or replace the ignored PostgreSQL cache tests with deterministic
  local evidence;
- document setup requirements without making a live PostgreSQL service a
  default fast-smoke dependency.

Before this adapter can move from `experimental` to `supported`, a later task
must additionally provide:

- fixture or isolated-container evidence for scoped persistence operations;
- failure-mode tests for missing rows, FK violations, and reconnect/close
  behavior;
- clear docs explaining which cache state is durable and which remains
  in-memory only;
- release-gate evidence approved by Adapter & Integration and Verification.

## Documentation Rule

Docs may mention PostgreSQL cache code only as unsupported or future work unless
a later task changes this classification with tests and evidence. Do not infer
PostgreSQL cache adapter support from the database administration CLI.
