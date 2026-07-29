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

Historical result after PAR-003: 23 active ignored Rust test attributes were
found. The current result after PAR-006 is 18.

Release gate fixture note after GH-RELEASE-PERSISTENCE-HIGH-PRECISION-FIXTURES:
the current direct `#[ignore]` count is 18, and there are 6 additional
high-precision-only `cfg_attr(..., ignore = "...")` test skips in
`crates/persistence/tests/test_catalog.rs`. These skips apply only when the
`high-precision` feature is enabled, because the legacy parquet fixtures encode
standard-precision 8-byte price fields while the release high-precision build
expects 16-byte fields. They are tracked below as `IGN-MED-009` and must not be
used as high-precision release evidence.

DRG-008 originally classified every High impact item. PAR-001 restored
`IGN-HIGH-003` and `IGN-HIGH-004`, PAR-002 restored `IGN-HIGH-005`, and
PAR-003 restored `IGN-HIGH-006` and `IGN-HIGH-007` to the default suite.
PAR-004 restored `IGN-HIGH-008` through `IGN-HIGH-010` at the current Strategy
routing owner. PAR-005 restored `IGN-HIGH-011` at the current AccountsManager
owner and corrected cash reservations to use order leaves quantity after a
partial fill. PAR-006 restored `IGN-HIGH-012` with a one-shot server disconnect
fixture and exact subscription replay assertions. The remaining active
ignored-test count is 18.

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

## DRG-008 High Impact Closure Result

DRG-008 closes the High impact `OPEN` queue by converting every remaining High
impact ignored test into a formal release-gate blocker. This is a strict
readiness result, not a runtime fix.

| Result | Count | Meaning |
| --- | ---: | --- |
| `BLOCKER_RECORDED` | 10 | At DRG-008 closeout, every remaining High impact ignored test was converted into a formal blocker. |
| Restored to default suite | 0 | DRG-008 was classification-only and did not repair runtime behavior. |

Current delta after DRG-008: PAR-001 through PAR-006 restored all ten High
impact blockers. No High impact ignored-test blocker remains active.

## V031-009 v0.3.1 Batch-1 Closure Result

V031-009 applies the first patch-release closure pass after the local
supervisor control-console hardening work. It does not mark the runtime,
adapter, or stress ignored tests as fixed. Instead it records whether the
`ntpro-rust-only-v0.3.1` release claim depends on each batch.

`v0.3.1` is scoped to Local Supervisor Control Console Hardening:

- local supervisor registry and process bookkeeping;
- local pause/resume/reconnect status semantics;
- Dashboard/API negative-path behavior;
- release-smoke wiring for the already scoped local sandbox control surface.

It explicitly does not claim production matching-engine parity, risk-engine
order-list/emulator routing parity, dYdX live reconnect support, or live-node
throughput/starvation performance guarantees. This was the historical
`v0.3.1` scope decision. PAR-001 through PAR-005 later restored the covered
matching-engine, reducing-order, emulator-routing, and account-balance tests;
dYdX reconnect and live performance behaviors remain blocked for future
adapter/performance hardening. None of those later restorations are retroactive
evidence for the `v0.3.1` patch release.

Batch-1 decision:

| Batch | Covered IDs | V031-009 decision | Why the v0.3.1 release claim does not rely on it |
| --- | --- | --- | --- |
| Execution/risk high-impact runtime blockers | `IGN-HIGH-003` through `IGN-HIGH-011` | `SCOPED_OUT_FOR_V031`; historical v0.3.1 decision. `IGN-HIGH-003` through `IGN-HIGH-007` were later restored by PAR-001 through PAR-003; `IGN-HIGH-008` through `IGN-HIGH-010` were later restored by PAR-004; `IGN-HIGH-011` was later restored by PAR-005. No execution/risk item in this historical group remains blocked. | `v0.3.1` does not advertise new trading-semantic, matching-engine, risk-engine, emulator, or account-balance behavior. The patch release is limited to local supervisor control-console hardening. |
| dYdX reconnect high-impact adapter blocker | `IGN-HIGH-012` | `SCOPED_OUT_FOR_V031`; historical v0.3.1 decision. The deterministic fixture was later restored by PAR-006. | `v0.3.1` reconnect controls are explicitly local sandbox `not_supported` results. They do not claim real venue reconnect or subscription replay. |
| Live stress/performance ignored tests | `IGN-MED-001`, `IGN-MED-002` | `RELEASE/PERF_ONLY`; deterministic v0.3 smoke covers the current local supervisor boundary. | `v0.3.1` does not claim live throughput or starvation-performance guarantees. The patch release uses `v03_supervisor_control_smoke.sh` and `v03_dashboard_smoke.sh` for deterministic local control evidence. |

Closure rule:

- These tests must not be counted as `v0.3.1` release evidence.
- They must remain visible in this register until a later task restores,
  replaces, or explicitly scopes each product behavior.
- `SCOPED_OUT_FOR_V031` is not `DONE`; it only prevents the patch release from
  accidentally depending on ignored trading/runtime/adapter evidence.

## V04-011 Binance Sandbox Closure Result

V04-011 applies the v0.4 Binance Sandbox Product Foundation closure pass. It
does not mark existing runtime, adapter, database, or stress ignored tests as
fixed. Instead it records that the v0.4 release claim does not depend on those
ignored tests.

The v0.4 claim is limited to:

- Binance fixture replay;
- mock Binance order lifecycle;
- deterministic halted-state risk rejection;
- EMA and RSI strategy smokes;
- local Dashboard exchange/strategy/order/risk panels.

The scoped v0.4 files were scanned for ignored tests:

```bash
rg -n "#\[ignore" crates/adapters/binance crates/trading/tests \
  crates/risk/tests/v04_binance_risk_rejection.rs crates/cli/src/dashboard.rs --glob '*.rs'
```

Result: no ignored tests were found in the scoped v0.4 evidence files.

V04-011 decision:

| Batch | Covered IDs | V04-011 decision | Why the v0.4 release claim does not rely on it |
| --- | --- | --- | --- |
| Execution matching-engine blockers | `IGN-HIGH-003`, `IGN-HIGH-004`, `IGN-HIGH-005` | `SCOPED_OUT_FOR_V04`; historical v0.4 decision. `IGN-HIGH-003` and `IGN-HIGH-004` were later restored by PAR-001; `IGN-HIGH-005` was restored by PAR-002. | v0.4 used deterministic mock Binance lifecycle evidence and did not claim production contingent/OCO/trailing-stop matching-engine behavior. |
| Broad risk-engine blockers | `IGN-HIGH-006` through `IGN-HIGH-011` | `SCOPED_OUT_FOR_V04`; historical v0.4 decision. `IGN-HIGH-006` and `IGN-HIGH-007` were later restored by PAR-003; `IGN-HIGH-008` through `IGN-HIGH-010` were later restored by PAR-004; `IGN-HIGH-011` was later restored by PAR-005. No risk-engine item in this historical group remains blocked. | v0.4 proves one halted-state Binance sandbox rejection through `V04-009`; it does not claim order-list reducing, emulator routing, or account-balance tracking. |
| PostgreSQL cache rejected-order tests | `IGN-MED-004`, `IGN-MED-005` | `SCOPED_OUT_FOR_V04`; still open for infrastructure hardening. | v0.4 uses local fixture/read-model evidence and does not claim durable PostgreSQL cache persistence. |
| Live stress/performance ignored tests | `IGN-MED-001`, `IGN-MED-002` | `RELEASE/PERF_ONLY_FOR_V04`; still manual/performance scoped. | v0.4 does not claim live-node throughput or cancellation-starvation performance guarantees. |

Closure rule:

- These tests must not be counted as `v0.4.0` release evidence.
- The v0.4 readiness report must cite the active V04 evidence tests instead.
- `SCOPED_OUT_FOR_V04` is not `DONE`; it only prevents the Binance sandbox
  release claim from accidentally depending on ignored runtime or integration
  evidence.

Formal blocker groups:

| Blocker ID | Covered ignored tests | Required follow-up |
| --- | --- | --- |
| `DRG8-BLOCKER-001` | `IGN-HIGH-003`, `IGN-HIGH-004` | Closed by PAR-001: matching-engine contingent/OUO decisions use validated post-update quantity without assuming synchronous cache event application. |
| `DRG8-BLOCKER-002` | `IGN-HIGH-005` | Closed by PAR-002: disabled trade execution still advances book-based iteration and LastPrice trailing maintenance without making the trade price an ordinary-limit execution source. |
| `DRG8-BLOCKER-003` | `IGN-HIGH-006`, `IGN-HIGH-007` | Closed by PAR-003: real filled Position fixtures initialize Portfolio state, fixed-precision inputs pass both precision modes, and executable risk traces prove all list members are denied without execution forwarding. |
| `DRG8-BLOCKER-004` | `IGN-HIGH-008`, `IGN-HIGH-009`, `IGN-HIGH-010` | Closed by PAR-004: current Strategy ownership routes bracket lists, single submits, and modifications of emulated orders to `OrderEmulator.execute`; default tests assert exact commands and zero RiskEngine forwarding. |
| `DRG8-BLOCKER-005` | `IGN-HIGH-011` | Closed by PAR-005: AccountsManager replays submitted, accepted, partial-fill, and final-fill events; assertions cover total, locked, free, and cumulative commission balances, including leaves-only reservation after the partial fill. |
| `DRG8-BLOCKER-006` | `IGN-HIGH-012` | Closed by PAR-006: the mock server consumes one disconnect request exactly once; the default regression asserts one replacement connection, one expected successful trade subscription replay, and an active client. |

## Classification

| Risk | Meaning | Default handling |
| --- | --- | --- |
| High | Product-reachable runtime behavior, execution/risk/portfolio/cache state, or adapter reconnect behavior may be incomplete. | Keep open, assign owner role, split into repair or explicit scope-decision tasks. |
| Medium | Important regression or integration evidence, but not suitable for every PR due speed, environment, or setup constraints. | Keep manual or release/perf scoped until a deterministic smaller test exists. |
| Low | One-time dataset curation, live API-key smoke, or external dependency check. | Keep manual; do not treat as product-gate evidence unless adapter scope requires it. |

## High Impact Register

| ID | Location | Ignored test | Reason recorded in source | Product path / impact | Owner role | Status | Recommended next step |
| --- | --- | --- | --- | --- | --- | --- | --- |
| IGN-HIGH-003 | `crates/execution/tests/matching_engine.rs` | `test_updating_of_contingent_orders` | Historical stale parent snapshot. | Contingent quantity propagation now uses validated post-update state. | Rust Core Runtime Agent | RESTORED_BY_PAR_001 | Default integration test defers cache application and asserts parent/child `OrderUpdated` quantity `2.000`. |
| IGN-HIGH-004 | `crates/execution/tests/matching_engine.rs` | `test_ouo_child_cancelled_when_parent_leaves_zero` | Historical stale parent snapshot. | OUO child cancellation now uses effective zero leaves. | Rust Core Runtime Agent | RESTORED_BY_PAR_001 | Default integration test asserts child cancellation before parent cancellation. |
| IGN-HIGH-005 | `crates/execution/tests/matching_engine.rs` | `test_trailing_stop_market_updated_then_triggered` | Historical L2 disabled-trade-execution maintenance gap. | Trailing stop update/trigger/fill now runs against the unchanged L2 book while LastPrice trails from trade ticks. | Rust Core Runtime Agent | RESTORED_BY_PAR_002 | Fixture-backed default regression asserts trigger `1485`, fill `1500`, and a negative ordinary-limit isolation case. |
| IGN-HIGH-006 | `crates/risk/tests/risk_engine.rs` | `test_submit_order_list_buys_when_trading_reducing_then_denies_orders` | Historical fixture never established the claimed LONG position. | BUY order-lists that would increase a LONG exposure are denied per member. | Rust Core Runtime Agent | RESTORED_BY_PAR_003 | Real filled LONG Position plus Portfolio initialization; default test and executable risk replay assert two denial events and zero execution forwarding. |
| IGN-HIGH-007 | `crates/risk/tests/risk_engine.rs` | `test_submit_order_list_sells_when_trading_reducing_then_denies_orders` | Historical fixture never established the claimed SHORT position; precision blocker was stale. | SELL order-lists that would increase a SHORT exposure are denied in both precision modes. | Rust Core Runtime Agent | RESTORED_BY_PAR_003 | Fixed-precision SHORT Position fixture; standard/high-precision tests and executable risk replay assert fail-closed behavior. |
| IGN-HIGH-008 | `crates/trading/src/strategy/mod.rs` | `test_submit_bracket_with_emulated_orders_sends_to_emulator` | Historical placeholder was owned by RiskEngine, but current strategy routing dispatches emulated lists before RiskEngine. | Bracket-order emulator routing is covered by a default test with a real emulated child. | Rust Core Runtime Agent | RESTORED_BY_PAR_004 | Assert exact list identity at `OrderEmulator.execute` and zero RiskEngine commands. |
| IGN-HIGH-009 | `crates/trading/src/strategy/mod.rs` | `test_submit_order_for_emulation_sends_command_to_emulator` | Historical placeholder was owned by RiskEngine, but current strategy routing dispatches emulated orders before RiskEngine. | Single-order emulator routing is covered by a default test with initialization-time metadata. | Rust Core Runtime Agent | RESTORED_BY_PAR_004 | Assert exact client order identity at `OrderEmulator.execute` and zero RiskEngine commands. |
| IGN-HIGH-010 | `crates/trading/src/strategy/mod.rs` | `test_modify_order_for_emulated_order_then_sends_to_emulator` | Historical placeholder was owned by RiskEngine, but current strategy routing dispatches modifications of `OrderStatus::Emulated` orders before RiskEngine. | Emulated modify routing is covered by a default test with a real `OrderEmulated` transition. | Rust Core Runtime Agent | RESTORED_BY_PAR_004 | Assert exact quantity/client order identity at `OrderEmulator.execute` and zero RiskEngine commands. |
| IGN-HIGH-011 | `crates/portfolio/src/manager.rs` | `test_partial_fill_and_full_fill_account_balance_correct` | Historical placeholder was attached to RiskEngine, while AccountsManager owns balance and reservation updates. | Partial and full cash fills now update total, locked, free, and cumulative commission balances under a default executable event sequence. | Rust Core Runtime Agent | RESTORED_BY_PAR_005 | The regression proves 80,000 initial reservation, 48,000 leaves-only reservation after a 40,000 partial fill, zero reservation after the final fill, and exact commission-adjusted balances. |
| IGN-HIGH-012 | `crates/adapters/dydx/tests/websocket.rs` | `test_subscription_restoration_tracking` | Historical mock used a level-triggered disconnect flag and closed replacement connections before replay completed. | dYdX reconnect and expected subscription replay now run in the default adapter suite without external credentials. | Adapter & Integration Agent | RESTORED_BY_PAR_006 | One-shot disconnect fixture; exact one reconnect, one successful `v4_trades` replay, and active-client assertions; repeated 20 times locally. |

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
| IGN-MED-009 | `crates/persistence/tests/test_catalog.rs` | 6 legacy parquet fixture read tests under `feature = "high-precision"` | Standard-precision parquet fixtures store 8-byte price fields; high-precision release builds expect 16-byte fixed precision. | High-precision release evidence must not rely on these legacy fixture reads. Generated high-precision catalog roundtrip coverage remains active. | Rust Core Runtime Agent | HIGH-PRECISION FIXTURE SCOPED OUT | Regenerate equivalent high-precision parquet fixtures or add a documented compatibility reader before using these fixture reads as release evidence. |

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
- `BLOCKER_RECORDED` is not `DONE`. It is a formal release-gate blocker and
  prevents product design from relying on that behavior until repaired or
  explicitly scoped out.
- Runtime-facing High items should be split before further runtime hardening
  work if they block a product path.
- External API-key and manual dataset tests should use mocks, fixtures, or
  sandbox evidence for release gates.
- Stress/performance tests should remain manual unless a deterministic small
  smoke is extracted for default verification.
