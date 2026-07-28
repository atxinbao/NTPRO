# Ignored Tests Risk Register

Date: 2026-06-11
Executor: Codex
Task: P1-006

## Purpose

This register tracks every current `#[ignore]` test attribute found under
`crates` and `tests`. P1-006 does not re-enable tests or change test behavior;
it turns ignored coverage into explicit follow-up work with owners and close
conditions.

## Summary

Command used:

```bash
rg -n "^\s*#\[ignore" crates tests
```

Current direct `#[ignore]` count after PAR-001: 26 ignored test attributes.

GH-RELEASE-PERSISTENCE-HIGH-PRECISION-FIXTURES note: there are also 6
high-precision-only `cfg_attr(..., ignore = "...")` fixture skips in
`crates/persistence/tests/test_catalog.rs`. They are not counted by the direct
`#[ignore]` scan above. They apply only to legacy standard-precision parquet
fixture reads under the `high-precision` feature, and are tracked as
`IGN-PERSIST-002`.

V031-009 note: the first v0.3.1 closure batch does not reduce this count. It
classifies the high-impact execution/risk/dYdX blockers and the live stress
tests against the `v0.3.1` Local Supervisor Control Console Hardening claim.
Those tests remain ignored, but they are explicitly not evidence for v0.3.1.

V04-011 note: the v0.4 Binance Sandbox Product Foundation does not depend on
any active `#[ignore]` test. The v0.4 product path is covered by default Rust
tests for Binance fixture replay, mock order lifecycle, risk rejection,
EMA/RSI strategy smokes, and Dashboard/API panels. The adjacent execution
matching-engine, broad risk-engine, PostgreSQL cache, and live stress ignored
tests remain tracked for future runtime or integration hardening, but they are
formally scoped out of the v0.4 Binance sandbox release claim.

## V04-011 Binance Sandbox Closure Result

Command used to check the scoped v0.4 files:

```bash
rg -n "#\[ignore" crates/adapters/binance crates/trading/tests \
  crates/risk/tests/v04_binance_risk_rejection.rs crates/cli/src/dashboard.rs --glob '*.rs'
```

Result: no ignored tests were found in the scoped v0.4 Binance sandbox evidence
files.

| v0.4 product path | Default test evidence | Ignored-test dependency |
| --- | --- | --- |
| Binance fixture replay | `crates/adapters/binance/tests/v04_replay.rs` | None |
| Mock order lifecycle | `crates/adapters/binance/tests/v04_mock_lifecycle.rs` | None |
| Risk rejection smoke | `crates/risk/tests/v04_binance_risk_rejection.rs` | None |
| EMA strategy smoke | `crates/trading/tests/v04_ema_smoke.rs` | None |
| RSI strategy smoke | `crates/trading/tests/v04_rsi_smoke.rs` | None |
| Dashboard business panels | `cargo test -p nautilus-cli dashboard --lib` | None |

V04-011 scoped-out set:

| Register IDs | V04 decision | Reason |
| --- | --- | --- |
| `IGN-EXEC-001` through `IGN-EXEC-003` | `SCOPED_OUT_FOR_V04`; historical v0.4 decision. `IGN-EXEC-001` and `IGN-EXEC-002` were later restored by PAR-001. | v0.4 used a deterministic mock Binance order lifecycle and did not claim production matching-engine contingent/OCO/trailing-stop semantics. |
| `IGN-RISK-001` through `IGN-RISK-006` | `SCOPED_OUT_FOR_V04`; still open for broader risk-engine hardening. | v0.4 proves one halted-state Binance sandbox rejection through `V04-009`; it does not claim order-list reducing, emulator routing, or account-balance tracking. |
| `IGN-INFRA-001`, `IGN-INFRA-002` | `SCOPED_OUT_FOR_V04`; PostgreSQL cache remains unsupported/open. | v0.4 dashboard and sandbox evidence are local fixture/read-model paths, not durable PostgreSQL cache persistence. |
| `IGN-LIVE-001`, `IGN-LIVE-002` | `RELEASE/PERF_ONLY_FOR_V04`; still manual/performance scoped. | v0.4 does not claim live-node throughput or cancellation-starvation performance guarantees. |

## Register

| ID | Test / scope | Path | Reason | Impact | Owner | Target | Close condition | State |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `IGN-RISK-001` | `test_submit_order_list_buys_when_trading_reducing_then_denies_orders` | `crates/risk/tests/risk_engine.rs:2911` | Portfolio state tracking integration missing. | Risk engine order-list reducing behavior lacks default regression coverage. | Rust Core Runtime Agent | v0.3 hardening | Implement portfolio state tracking fixture or scope out with replacement test. | Scoped out for v0.3.1 and v0.4 Binance sandbox; open for runtime hardening. |
| `IGN-RISK-002` | `test_submit_order_list_sells_when_trading_reducing_then_denies_orders` | `crates/risk/tests/risk_engine.rs:3052` | High-precision decimal dependency noted. | Risk engine sell-side reducing behavior lacks default regression coverage. | Rust Core Runtime Agent | v0.3 hardening | Re-run with current precision mode; unignore if passing or replace with stable precision fixture. | Scoped out for v0.3.1 and v0.4 Binance sandbox; open for runtime hardening. |
| `IGN-RISK-003` | `test_submit_bracket_with_emulated_orders_sends_to_emulator` | `crates/risk/tests/risk_engine.rs:3204` | Emulator integration missing. | Bracket-order emulation risk path is unverified by default. | Rust Core Runtime Agent | v0.3 hardening | Add emulator integration or mark emulated order flow out of product scope. | Scoped out for v0.3.1 and v0.4 Binance sandbox; open for runtime hardening. |
| `IGN-RISK-004` | `test_submit_order_for_emulation_sends_command_to_emulator` | `crates/risk/tests/risk_engine.rs:3314` | Emulator integration missing. | Emulated order command path is unverified by default. | Rust Core Runtime Agent | v0.3 hardening | Add emulator integration test or replacement smoke. | Scoped out for v0.3.1 and v0.4 Binance sandbox; open for runtime hardening. |
| `IGN-RISK-005` | `test_modify_order_for_emulated_order_then_sends_to_emulator` | `crates/risk/tests/risk_engine.rs:3522` | Emulator integration missing. | Emulated modify path is unverified by default. | Rust Core Runtime Agent | v0.3 hardening | Add emulator modify fixture or mark unsupported. | Scoped out for v0.3.1 and v0.4 Binance sandbox; open for runtime hardening. |
| `IGN-RISK-006` | `test_partial_fill_and_full_fill_account_balance_correct` | `crates/risk/tests/risk_engine.rs:3876` | Account balance tracking missing. | Fill/account-balance risk path is unverified by default. | Rust Core Runtime Agent | v0.3 hardening | Add account balance tracking implementation and unignore, or replace with supported behavior test. | Scoped out for v0.3.1 and v0.4 Binance sandbox; open for runtime hardening. |
| `IGN-EXEC-001` | `test_updating_of_contingent_orders` | `crates/execution/tests/matching_engine.rs` | Historical stale parent snapshot. | Contingent quantity propagation now has default regression coverage. | Rust Core Runtime Agent | PAR-001 | Read the current cached parent snapshot before the OUO decision. | Restored to default suite by PAR-001. |
| `IGN-EXEC-002` | `test_ouo_child_cancelled_when_parent_leaves_zero` | `crates/execution/tests/matching_engine.rs` | Historical stale parent snapshot. | Zero-leaves child cancellation now has default regression coverage. | Rust Core Runtime Agent | PAR-001 | Read the current cached parent snapshot before the OUO decision. | Restored to default suite by PAR-001. |
| `IGN-EXEC-003` | `test_trailing_stop_market_updated_then_triggered` | `crates/execution/tests/matching_engine.rs:6633` | L2 engine/trade_execution behavior noted as unresolved. | Trailing stop update/trigger path lacks default regression coverage. | Rust Core Runtime Agent | v0.3 hardening | Clarify product support and unignore or replace with supported path test. | Scoped out for v0.3.1 and v0.4 Binance sandbox; open for runtime hardening. |
| `IGN-INFRA-001` | `test_order_cancel_rejected_insert_and_load` | `crates/infrastructure/tests/test_cache_postgres.rs:184` | PostgreSQL schema FK constraints incomplete. | PostgreSQL cache rejected cancel persistence is unverified by default. | Adapter & Integration Agent | v0.3 hardening | Complete schema/FK support or mark PostgreSQL cache unsupported in product docs. | Scoped out for v0.4 Binance sandbox; PostgreSQL cache remains unsupported/open. |
| `IGN-INFRA-002` | `test_order_modify_rejected_insert_and_load` | `crates/infrastructure/tests/test_cache_postgres.rs:240` | PostgreSQL schema FK constraints incomplete. | PostgreSQL cache rejected modify persistence is unverified by default. | Adapter & Integration Agent | v0.3 hardening | Complete schema/FK support or mark PostgreSQL cache unsupported in product docs. | Scoped out for v0.4 Binance sandbox; PostgreSQL cache remains unsupported/open. |
| `IGN-LIVE-001` | `stress_trade_burst` | `crates/live/tests/stress.rs:263` | Stress test excluded from routine default tests. | Live-node burst throughput regression is manual only. | Rust Core Runtime Agent | v0.3 hardening | Keep ignored with documented manual cadence or split a smaller default smoke. | Release/perf only for v0.3.1 and v0.4; deterministic local supervisor/dashboard smokes cover the current boundary. |
| `IGN-LIVE-002` | `stress_cancel_starvation` | `crates/live/tests/stress.rs:337` | Stress test excluded from routine default tests. | Live-node cancellation starvation regression is manual only. | Rust Core Runtime Agent | v0.3 hardening | Keep ignored with documented manual cadence or split a smaller default smoke. | Release/perf only for v0.3.1 and v0.4; deterministic local supervisor/dashboard smokes cover the current boundary. |
| `IGN-PLUGIN-001` | `loads_example_cdylib_and_walks_manifest` | `crates/plugin/tests/load_example_cdylib.rs:102` | Example cdylib smoke kept out of default test path. | Plug-in load integration is manual only. | Verification & Release Gatekeeper | v0.3 hardening | Keep an explicit release smoke command or make a platform-stable default test. | Open |
| `IGN-PERSIST-001` | `test_write_data_enum_mixed_custom_data_identifiers` | `crates/persistence/tests/test_catalog.rs:3685` | Slow custom-data regression over 120s. | Custom data identifier batching regression is manual only. | Rust Core Runtime Agent | v0.3 hardening | Reduce runtime or move to scheduled/release-only validation with evidence. | Open |
| `IGN-PERSIST-002` | 6 legacy parquet fixture read tests under `feature = "high-precision"` | `crates/persistence/tests/test_catalog.rs` | Existing fixture files encode standard-precision 8-byte price fields; high-precision decode expects 16-byte fixed precision. | Release high-precision gate must use generated high-precision roundtrip tests instead of these legacy fixture reads. | Rust Core Runtime Agent | release gate hardening | Regenerate high-precision fixture equivalents or add an explicit compatibility reader before treating these fixtures as high-precision release evidence. | Scoped out for high-precision release builds; active in standard-precision builds. |
| `IGN-DYDX-001` | `test_subscription_restoration_tracking` | `crates/adapters/dydx/tests/websocket.rs:1408` | Server-triggered disconnect causes reconnect loop. | dYdX reconnect/subscription replay gap remains tracked but not default-covered. | Adapter & Integration Agent | v0.3 hardening | Fix reconnect trigger reset or replace with deterministic fixture. | Scoped out for v0.3.1; open for adapter hardening. |
| `IGN-HL-001` | `test_connect_times_out_when_account_never_registers` | `crates/adapters/hyperliquid/tests/exec_client.rs:5004` | Hard-coded account registration timeout blocks about 30s. | Hyperliquid connect timeout behavior is manual only. | Adapter & Integration Agent | v0.3 hardening | Make timeout injectable and unignore or add faster fixture. | Open |
| `IGN-BLOCK-001` | `pool_snapshot_request_does_not_emit_snapshot_when_bootstrap_fails` | `crates/adapters/blockchain/src/data/client.rs:1024` | Requires `ENVIO_API_TOKEN` and live HyperSync access. | Blockchain live bootstrap failure path is external-service gated. | Adapter & Integration Agent | v0.3 adapter scope | Replace with fixture/mock or keep documented as live-only manual test. | Open |
| `IGN-BLOCK-002` | `live_hypersync_bootstrap_fails_closed_when_rpc_hydration_fails` | `crates/adapters/blockchain/src/data/core.rs:1471` | Requires `ENVIO_API_TOKEN` and live HyperSync access. | Blockchain RPC hydration failure path is external-service gated. | Adapter & Integration Agent | v0.3 adapter scope | Replace with fixture/mock or keep documented as live-only manual test. | Open |
| `IGN-BYBIT-001` | `test_request_tickers_spot_live` | `crates/adapters/bybit/tests/http.rs:2199` | Requires real Bybit API access. | Bybit spot live HTTP behavior is not default-covered. | Adapter & Integration Agent | v0.3 adapter scope | Replace with recorded/mock fixture or document as manual live test. | Open |
| `IGN-BYBIT-002` | `test_request_tickers_linear_live` | `crates/adapters/bybit/tests/http.rs:2267` | Requires real Bybit API access. | Bybit linear live HTTP behavior is not default-covered. | Adapter & Integration Agent | v0.3 adapter scope | Replace with recorded/mock fixture or document as manual live test. | Open |
| `IGN-BYBIT-003` | `test_request_tickers_inverse_live` | `crates/adapters/bybit/tests/http.rs:2360` | Requires real Bybit API access. | Bybit inverse live HTTP behavior is not default-covered. | Adapter & Integration Agent | v0.3 adapter scope | Replace with recorded/mock fixture or document as manual live test. | Open |
| `IGN-BYBIT-004` | `test_request_tickers_with_symbol_filter` | `crates/adapters/bybit/tests/http.rs:2411` | Requires real Bybit API access. | Bybit symbol-filter live HTTP behavior is not default-covered. | Adapter & Integration Agent | v0.3 adapter scope | Replace with recorded/mock fixture or document as manual live test. | Open |
| `IGN-BITMEX-001` | `test_rate_limiting` | `crates/adapters/bitmex/tests/http.rs:527` | Slow integration test. | BitMEX rate-limit behavior is manual/slow only. | Adapter & Integration Agent | v0.3 adapter scope | Make rate limiter clock injectable or keep as release-only slow test with evidence. | Open |
| `IGN-BETFAIR-001` | `test_load_match_odds_file` | `crates/adapters/betfair/src/loader.rs:670` | Requires user-fetched local Betfair data. | Betfair match odds loader path is fixture-user gated. | Adapter & Integration Agent | v0.3 adapter scope | Add small committed fixture or document manual fixture requirement. | Open |
| `IGN-BETFAIR-002` | `test_load_racing_win_file` | `crates/adapters/betfair/src/loader.rs:731` | Requires user-fetched local Betfair data. | Betfair racing win loader path is fixture-user gated. | Adapter & Integration Agent | v0.3 adapter scope | Add small committed fixture or document manual fixture requirement. | Open |
| `IGN-TARDIS-001` | `test_curate_deribit_deltas` | `crates/adapters/tardis/src/csv/load.rs:1409` | One-time dataset curation. | Tardis large CSV curation is manual only. | Adapter & Integration Agent | v0.3 adapter scope | Keep as manual curation or add a smaller default fixture. | Open |
| `IGN-TESTKIT-001` | `test_curate_aapl_itch` | `crates/testkit/src/itch/parse.rs:913` | One-time dataset curation. | ITCH large dataset curation is manual only. | Verification & Release Gatekeeper | v0.3 testing | Keep as manual curation or add a smaller default fixture. | Open |

## Policy

- New `#[ignore]` tests should include an owner, reason, close condition, and
  target milestone in this register.
- Production-path ignored tests should be either unignored, replaced with a
  deterministic fixture, or explicitly scoped out before release claims depend
  on that behavior.
- External-service tests should not require live credentials in routine CI; use
  recorded fixtures, mocks, sandbox endpoints, or documented manual commands.
