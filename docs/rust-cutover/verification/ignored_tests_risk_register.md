# Ignored Tests Risk Register

Date: 2026-06-05
Executor: Codex
Task ID: GH-160

## Purpose

This register records ignored Rust tests that can hide product, runtime,
adapter, data, persistence, or verification risk. It does not mark any ignored
test as fixed. It is a triage document for deciding which ignored tests should
be promoted, repaired, kept manual, or scoped out in later work.

## Scan Scope

Commands used:

```bash
rg -n '^\s*#\[ignore(?:\s*=\s*"[^"]*")?\]' crates tests --glob '*.rs' -S
rg -n '^\s*#\[ignore(?:\s*=\s*"[^"]*")?\]' crates tests --glob '*.rs' -S | wc -l
rg -n '^\s*#\[ignore(?:\s*=\s*"[^"]*")?\]' crates tests --glob '*.rs' -S | cut -d: -f1 | sort | uniq -c
```

Result after NAUDIT-003: 28 active ignored Rust test attributes were found.

NAUDIT-003 restored two previously ignored common cache lifecycle tests to the
default Rust test suite. They are no longer active ignored production-bug
entries:

| Restored ID | Location | Test | Restoration evidence |
| --- | --- | --- | --- |
| IGN-HIGH-001 | `crates/common/src/cache/tests.rs` | `test_order_when_rejected` | Restored by `NAUDIT-003`; runs without `--ignored`. |
| IGN-HIGH-002 | `crates/common/src/cache/tests.rs` | `test_order_when_filled` | Restored by `NAUDIT-003`; runs without `--ignored`. |

`Cargo.toml` `ignored = [...]` feature declarations were scanned separately.
Those are feature-workspace declarations, not ignored tests, and are not counted
in this register.

## Classification

| Risk | Meaning | Default handling |
| --- | --- | --- |
| High | Product-reachable runtime behavior, execution/risk/portfolio/cache state, or adapter reconnect behavior may be incomplete. | Keep open, assign owner role, split into repair or explicit scope-decision tasks. |
| Medium | Important regression or integration evidence, but not suitable for every PR due speed, environment, or setup constraints. | Keep manual or release/perf scoped until a deterministic smaller test exists. |
| Low | One-time dataset curation, live API-key smoke, or external dependency check. | Keep manual; do not treat as product-gate evidence unless adapter scope requires it. |

## High Impact Register

| ID | Location | Ignored test | Reason recorded in source | Product path / impact | Owner role | Status | Recommended next step |
| --- | --- | --- | --- | --- | --- | --- | --- |
| IGN-HIGH-003 | `crates/execution/tests/matching_engine.rs:3604` | `test_updating_of_contingent_orders` | Contingent-order helper reads parent leaves quantity from stale local clone. | Contingent OUO/OCO matching behavior can be stale after parent updates. | Rust Core Runtime Agent | OPEN | Refactor matching helper to read current cache/order handle and add regression coverage. |
| IGN-HIGH-004 | `crates/execution/tests/matching_engine.rs:4175` | `test_ouo_child_cancelled_when_parent_leaves_zero` | Same stale parent leaves quantity issue. | OUO child cancellation may be wrong when parent leaves quantity reaches zero. | Rust Core Runtime Agent | OPEN | Resolve with IGN-HIGH-003 or split if cancellation semantics differ. |
| IGN-HIGH-005 | `crates/execution/tests/matching_engine.rs:6443` | `test_trailing_stop_market_updated_then_triggered` | L2 engine with `trade_execution=false` does not iterate on trade ticks. | Trailing stop trigger behavior can be incomplete for L2 simulated execution. | Rust Core Runtime Agent | OPEN | Decide whether to implement L2 trade-tick iteration or scope trailing stop behavior out of the current product path. |
| IGN-HIGH-006 | `crates/risk/tests/risk_engine.rs:2911` | `test_submit_order_list_buys_when_trading_reducing_then_denies_orders` | Requires portfolio state tracking integration. | Risk rejection for order-list reducing behavior depends on portfolio state. | Rust Core Runtime Agent | OPEN | Add portfolio state tracking fixture or scope order-list reducing behavior as deferred. |
| IGN-HIGH-007 | `crates/risk/tests/risk_engine.rs:3052` | `test_submit_order_list_sells_when_trading_reducing_then_denies_orders` | Waiting on high-precision decimal merge. | High-precision risk/order-list reduction behavior remains unproven. | Rust Core Runtime Agent | OPEN | Re-run under high-precision path and either repair precision handling or downgrade with evidence. |
| IGN-HIGH-008 | `crates/risk/tests/risk_engine.rs:3204` | `test_submit_bracket_with_emulated_orders_sends_to_emulator` | Waiting on emulator implementation. | Bracket-order risk-to-emulator routing is not release-proven. | Rust Core Runtime Agent | OPEN | Tie to order emulator integration task; add mock emulator fixture. |
| IGN-HIGH-009 | `crates/risk/tests/risk_engine.rs:3314` | `test_submit_order_for_emulation_sends_command_to_emulator` | Waiting on emulator implementation. | Order emulation command routing is not release-proven. | Rust Core Runtime Agent | OPEN | Close with emulator integration evidence or mark emulator path deferred. |
| IGN-HIGH-010 | `crates/risk/tests/risk_engine.rs:3522` | `test_modify_order_for_emulated_order_then_sends_to_emulator` | Waiting on emulator implementation. | Modify-order path for emulated orders is not release-proven. | Rust Core Runtime Agent | OPEN | Close with emulator integration evidence or mark emulator path deferred. |
| IGN-HIGH-011 | `crates/risk/tests/risk_engine.rs:3876` | `test_partial_fill_and_full_fill_account_balance_correct` | Waiting on account balance tracking implementation. | Partial/full fill accounting and balance tracking can affect risk and portfolio correctness. | Rust Core Runtime Agent | OPEN | Add account balance tracking fixture and pair with portfolio/risk regression. |
| IGN-HIGH-012 | `crates/adapters/dydx/tests/websocket.rs:1408` | `test_subscription_restoration_tracking` | Server-triggered disconnect causes reconnect loop during subscription replay. | Adapter reconnect/replay behavior is not robust for dYdX websocket. | Adapter & Integration Agent | OPEN | Make disconnect trigger injectable or isolate reconnect loop with deterministic fixture. |

## Medium Impact Register

| ID | Location | Ignored test | Reason recorded in source | Product path / impact | Owner role | Status | Recommended next step |
| --- | --- | --- | --- | --- | --- | --- | --- |
| IGN-MED-001 | `crates/live/tests/stress.rs:263` | `stress_trade_burst` | Stress test ignored so default Cargo tests stay fast; should use release/perf mode. | Live message-bus throughput evidence is not part of default PR gate. | Verification & Release Gatekeeper | RELEASE/PERF ONLY | Keep ignored; define a smaller deterministic live smoke if product gate needs it. |
| IGN-MED-002 | `crates/live/tests/stress.rs:337` | `stress_cancel_starvation` | Stress test ignored so default Cargo tests stay fast; should use release/perf mode. | Cancel starvation under live trade pressure is not part of default PR gate. | Verification & Release Gatekeeper | RELEASE/PERF ONLY | Keep ignored; run only under release/perf evidence or derive deterministic smoke. |
| IGN-MED-003 | `crates/persistence/tests/test_catalog.rs:3685` | `test_write_data_enum_mixed_custom_data_identifiers` | Slow regression test over 120 seconds for custom data identifier batching. | Data/catalog custom-data batching is not part of fast verification. | Adapter & Integration Agent | MANUAL/SLOW | Keep manual until a smaller custom-data fixture can cover identifier grouping. |
| IGN-MED-004 | `crates/infrastructure/tests/test_cache_postgres.rs:184` | `test_order_cancel_rejected_insert_and_load` | Waiting on PostgreSQL schema completion and FK constraints. | Persistence of rejected cancel order state is not fully covered. | Adapter & Integration Agent | OPEN | NAUDIT-005 classifies PostgreSQL cache adapter as unsupported for v0.2; restore only after schema/FK and fixture evidence exist. |
| IGN-MED-005 | `crates/infrastructure/tests/test_cache_postgres.rs:240` | `test_order_modify_rejected_insert_and_load` | Waiting on PostgreSQL schema completion and FK constraints. | Persistence of rejected modify order state is not fully covered. | Adapter & Integration Agent | OPEN | NAUDIT-005 classifies PostgreSQL cache adapter as unsupported for v0.2; restore only after schema/FK and fixture evidence exist. |
| IGN-MED-006 | `crates/adapters/hyperliquid/tests/exec_client.rs:5004` | Hyperliquid account registration timeout test | Blocks about 30 seconds on hard-coded timeout. | Hyperliquid execution registration failure handling is not default-gated. | Adapter & Integration Agent | OPEN | Make timeout injectable and move to deterministic adapter fixture. |
| IGN-MED-007 | `crates/adapters/bitmex/tests/http.rs:527` | `test_rate_limiting` | Slow integration test around eight seconds. | BitMEX rate limiter behavior is not part of default PR gate. | Adapter & Integration Agent | MANUAL/SLOW | Keep manual unless adapter support matrix requires default rate-limit evidence. |
| IGN-MED-008 | `crates/plugin/tests/load_example_cdylib.rs:102` | `loads_example_cdylib_and_walks_manifest` | Plain Cargo tests stay fast; required Linux cdylib smoke uses a make target. | Plugin cdylib loading is outside default Rust-only product smoke. | Verification & Release Gatekeeper | MANUAL/PLATFORM | Keep manual platform smoke; do not treat as core CLI blocker. |

## Low / External Dependency Register

| ID | Location | Count | Reason | Owner role | Status | Recommended next step |
| --- | --- | ---: | --- | --- | --- | --- |
| IGN-LOW-001 | `crates/adapters/bybit/tests/http.rs` | 4 | Requires real Bybit API access. | Adapter & Integration Agent | EXTERNAL/API KEY | Keep ignored; prefer mocks, recorded fixtures, or sandbox dry-run evidence for product gates. |
| IGN-LOW-002 | `crates/adapters/blockchain/src/data/client.rs`, `crates/adapters/blockchain/src/data/core.rs` | 2 | Requires `ENVIO_API_TOKEN` and live HyperSync access. | Adapter & Integration Agent | EXTERNAL/API KEY | Keep ignored; replace product evidence with fixtures or mock hydration. |
| IGN-LOW-003 | `crates/adapters/betfair/src/loader.rs` | 2 | Requires user-fetched local Betfair data. | Adapter & Integration Agent | MANUAL/DATASET | Keep manual; use committed or generated fixture for product evidence. |
| IGN-LOW-004 | `crates/adapters/tardis/src/csv/load.rs` | 1 | One-time dataset curation, not routine CI. | Adapter & Integration Agent | MANUAL/DATASET | Keep manual curation; product evidence should use small fixture. |
| IGN-LOW-005 | `crates/testkit/src/itch/parse.rs` | 1 | One-time dataset curation, not routine CI. | Verification & Release Gatekeeper | MANUAL/DATASET | Keep manual curation; product evidence should use small fixture. |

## Follow-Up Policy

- Do not remove ignored tests to make verification green.
- Do not mark High impact ignored tests as resolved without a passing test,
  explicit scope decision, or release-gate deferral.
- Runtime-facing High items should be split before further runtime hardening
  work if they block a product path.
- External API-key and manual dataset tests should use mocks, fixtures, or
  sandbox evidence for release gates.
- Stress/performance tests should remain manual unless a deterministic small
  smoke is extracted for default verification.
