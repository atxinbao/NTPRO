# GH-158 Todo Panic Scope Decision

Date: 2026-06-05
Executor: Codex

## Task

- GitHub issue: <https://github.com/atxinbao/NTPRO/issues/158>
- Local task file: not present; this is a GitHub audit issue execution.
- Risk level: Medium

## Decision

Product-reachable or adapter-reachable `todo!()` / `unimplemented!()` panics
must not remain as runtime panic paths. When the current Rust-only product does
not implement the operation, the code should return an explicit unsupported or
not-implemented error instead.

## Replaced

The following active paths were changed from panic macros to explicit errors or
non-panicking unsupported responses:

- PostgreSQL cache adapter unsupported cache operations in
  `crates/infrastructure/src/sql/cache.rs`.
- Redis cache adapter unsupported cache operations in
  `crates/infrastructure/src/redis/cache.rs`.
- PostgreSQL row decoders for unsupported instrument/order model rows in
  `crates/infrastructure/src/sql/models/instruments.rs` and
  `crates/infrastructure/src/sql/models/orders.rs`.
- Blockchain RPC default swap subscribe/unsubscribe methods in
  `crates/adapters/blockchain/src/rpc/mod.rs`.
- Blockchain execution client unsupported execution/report commands in
  `crates/adapters/blockchain/src/execution/client.rs`.

## Classified As Deferred

These operations remain functionally deferred. This PR does not implement their
runtime behavior:

- Redis generic/account/order/position write paths that are not currently
  supported by the Redis cache adapter.
- PostgreSQL synthetic/position/order-book/state snapshot/index paths that are
  not currently supported by the PostgreSQL cache adapter.
- SQL row decoding for specific order events and spread instruments that still
  need dedicated schema/decoder work.
- Blockchain DeFi execution order commands and report generation.
- Blockchain RPC swap subscriptions.

## Reasonable Remaining Matches

After cleanup, remaining `todo!()` / `unimplemented!()` search hits are not
treated as product runtime panic blockers for this task:

- `crates/analysis/src/analyzer.rs`: test-local `MockAccount` helper methods
  inside the analyzer test module.
- `crates/persistence/src/backend/catalog.rs`: doctest-style placeholder
  comments using `unimplemented!()`.
- `crates/execution/src/order_manager/manager.rs`: historical doc comment
  mentioning a previous `todo!()` panic.

## Non-Goals

- No SQL/Redis/cache feature implementation.
- No blockchain DeFi execution implementation.
- No trading semantic change.
- No external API call.
- No deletion of tests or ignored tests.

## Follow-Up

Deferred cache, SQL decoder, and blockchain adapter behavior should be split
into separate implementation tasks only when the product support matrix marks
those paths as supported.
