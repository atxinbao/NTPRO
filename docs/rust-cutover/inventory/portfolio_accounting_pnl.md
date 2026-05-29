# Rust Portfolio / Accounting / PnL Gap Inventory

Date: 2026-05-29
Last updated: 2026-05-30 by Codex for RCORE-015
Executor: Codex
Task ID: RCORE-013, RCORE-015

## Scope

This inventory covers the Rust portfolio, account, margin, balance, realized
PnL, unrealized PnL, equity, and snapshot surfaces that affect the Rust-only
cutover gate.

The task is inventory-only. It does not change runtime behavior, trading
semantics, public APIs, adapters, persistence formats, or Python/PyO3/Cython
surfaces.

## Rust Surfaces Inspected

| Area | Files | Current Rust surface |
| --- | --- | --- |
| Portfolio runtime | `crates/portfolio/src/portfolio.rs` | `Portfolio` tracks account state, net positions, realized/unrealized PnL, mark values, equity, snapshots, missing-price state, and message-bus handlers for account, order, position, quote, bar, and mark-price updates. |
| Account mutation manager | `crates/portfolio/src/manager.rs` | `AccountsManager` updates balances from fills, order locks, margin init, margin maintenance, generated `AccountState`, and exchange-rate conversion. |
| Account model | `crates/model/src/accounts/{base,cash,margin,betting}.rs` | Rust account types expose cash/margin/betting balances, commissions, locked balances, margin balances, PnL calculations, account state apply, and event retention. |
| Portfolio events | `crates/model/src/events/portfolio/snapshot.rs` | `PortfolioSnapshot` carries account balances, margins, realized PnL, unrealized PnL, and total equity. |
| Python/PyO3 bridge | `crates/portfolio/src/python/mod.rs`, `crates/model/src/python/account/**`, `crates/model/src/python/events/portfolio/**` | PyO3 wrappers still expose portfolio/accounting APIs to Python. These are not final Rust-only product surfaces. |
| Legacy Cython/Python surface | `nautilus_trader/portfolio/**`, `nautilus_trader/accounting/**`, `tests/unit_tests/portfolio/**`, `tests/unit_tests/accounting/**` | Legacy Python/Cython accounting and portfolio paths still exist as comparison/migration surfaces and must not be deleted by RCORE-013. |

## Current Rust Evidence

| Behavior | Evidence |
| --- | --- |
| Cash and margin account balance accessors and account state apply | `crates/model/src/accounts/cash.rs` and `crates/model/src/accounts/margin.rs` unit tests cover balances, events, commissions, margin accessors, account-wide margins, and apply behavior. |
| Cash account fill PnL and balance locks | `crates/model/src/accounts/cash.rs` covers cash PnL and lock calculations; `crates/portfolio/src/manager.rs` covers multi-order locks, stale lock clearing, negative-balance rejection, borrowing behavior, and order-cancel lock release. |
| Margin initial and maintenance margin math | `crates/model/src/accounts/margin.rs` covers leverage, inverse instruments, option/binary-option premium PnL, and margin accessors; `crates/portfolio/src/manager.rs` covers net hedging maintenance margin, net-flat clearing, flip pricing, deterministic leg ordering, and unavailable xrate handling. |
| Portfolio realized/unrealized PnL queries | `crates/portfolio/tests/portfolio.rs` covers missing account/instrument behavior, realized PnL with snapshots, multiple snapshot paths, account-id filters, and missing xrate fallback. |
| Portfolio mark value and equity | `crates/portfolio/tests/portfolio.rs` covers cash equity, margin equity with unrealized PnL, account-balance currency order, short-position equity, foreign-settlement conversion, missing-price tracking, and missing-xrate tracking. |
| Portfolio snapshot model | `crates/portfolio/tests/portfolio.rs` covers `build_snapshot`, margin snapshot sourcing from cached account state, snapshot timers, message-bus publication, snapshot ring append, and timer cancellation. |

## Gap Matrix

| Gap | Status | Evidence | Release impact |
| --- | --- | --- | --- |
| PAPL-001 | Open: no executable `portfolio_pnl` golden trace replay. | `docs/rust-cutover/golden_trace/SCHEMA.md` defines `portfolio_pnl`, and `docs/rust-cutover/golden_trace/GATE_EVIDENCE.md` states there is no executable account balance, margin, realized PnL, unrealized PnL, or equity replay yet. No `tests/golden/*portfolio*` fixture exists. | Blocks final Rust-only release until a deterministic trace drives Rust portfolio/accounting code and compares account/equity/PnL outputs. |
| PAPL-002 | Open: Rust portfolio/accounting is still exposed through Python/PyO3 and legacy Cython/Python modules remain present. | `crates/portfolio/Cargo.toml` has `python` and `extension-module` features; `crates/portfolio/src/python/mod.rs` exposes `PyPortfolio`; `nautilus_trader/portfolio/**` and `nautilus_trader/accounting/**` remain present. | Blocks Python/PyO3/Cython removal until RREM/release gates approve deletion after Rust parity evidence exists. |
| PAPL-003 | Partial: RCORE-014 adds a Rust integration test from order fill replay through account balance, realized PnL, unrealized PnL, equity, and `PortfolioSnapshot`; no executable golden trace exists yet. | `crates/portfolio/tests/portfolio.rs::test_order_fill_replay_updates_balance_pnl_equity_and_snapshot` covers an open/close fill path with commissions, realized PnL, flat unrealized PnL, account equity, and snapshot totals. The golden trace gate still has no `portfolio_pnl` replay. | RTRACE follow-up must promote this coverage into a deterministic release-gate trace. |
| PAPL-004 | Closed by RCORE-015: Rust account trait implementations now report the stored calculated-state flag. | `CashAccount::calculated_account_state` and `MarginAccount::calculated_account_state` return `base.calculate_account_state`; `BettingAccount` already returned the same base flag. RCORE-015 added constructor-flag tests for cash and margin accounts and an `AccountAny::set_calculate_account_state` regression test covering cash, margin, and betting variants. | No release blocker remains for the trait flag. Generated `AccountState` runtime behavior is unchanged; this only fixes the Rust trait reporting surface. |
| PAPL-005 | Explicitly scoped by RCORE-015: keep current Python-parity graceful-degradation behavior until a dedicated typed accounting-result contract is approved. | `AccountsManager::update_balance_single_currency` still returns early on missing xrate or balance; `update_balance_multi_currency` still rejects missing negative debit currencies; `Portfolio::calculate_realized_pnl` can still return zero when xrate is unavailable and marks pending calculations. Existing tests continue to cover these paths, and RCORE-015 did not change exchange-rate, missing-balance, or PnL fallback semantics. | Not a blocker for RCORE-015. A future runtime/scope task must define whether NTPRO keeps Python-parity fallback semantics or introduces a Rust-only typed accounting result. |
| PAPL-006 | Partial: snapshot/equity is available in Rust and RCORE-014 covers a closed-position snapshot path, but not release replay evidence. | `Portfolio::build_snapshot`, `Portfolio::snapshots`, snapshot timers, and `PortfolioSnapshot` exist and have tests. `test_order_fill_replay_updates_balance_pnl_equity_and_snapshot` verifies snapshot balance, realized PnL, unrealized PnL, and total equity after a closed position. The trace gate still lacks executable `portfolio_pnl` evidence. | Release gate needs deterministic snapshot output comparison, not only unit tests. |
| PAPL-007 | Partial: multi-account and multi-venue aggregation has targeted Rust tests but no gate trace. | `crates/portfolio/tests/portfolio.rs` covers account-id filters for net exposure, unrealized PnL, realized PnL, and multiple account/equity cases. | Needs release-level trace or explicit scope decision for supported multi-account/multi-venue workflows. |
| PAPL-008 | Open: adapter/live account-state sources are not proven through portfolio PnL replay. | Adapter tests and Python integration tests include account/balance payloads, but RCORE-013 did not find a Rust `portfolio_pnl` trace that starts from adapter or live account-state events and verifies portfolio outputs. | Adapter/live account-state parity remains owned by later RBTL/RADP/RTRACE tasks. |

## Non-Goals Preserved

- No Python, PyO3, Cython, `build.py`, or `pyproject.toml` removal.
- No public API changes.
- No trading-semantic changes.
- No changes to margin, PnL, equity, snapshot, adapter, or persistence behavior.
- RCORE-015 changes only the Rust account trait reporting surface for the
  existing calculated-account-state flag.

## RCORE-015 Closure Decisions

| Area | Decision | Evidence |
| --- | --- | --- |
| Account calculated-state flag | Implemented in Rust account trait implementations. | `crates/model/src/accounts/cash.rs`, `crates/model/src/accounts/margin.rs`, and `crates/model/src/accounts/any.rs` regression tests. |
| Golden trace replay | Deferred to RTRACE/release-gate work. | PAPL-001, PAPL-003, PAPL-006, and PAPL-007 require executable `portfolio_pnl` trace evidence rather than more unit-only runtime changes. |
| Python/PyO3/Cython exposure | Deferred to RREM/removal gate. | PAPL-002 remains blocked until Rust product, runtime, adapter, and trace gates approve removal. |
| Missing xrate/balance typed result | Explicitly scoped out of RCORE-015. | PAPL-005 preserves current tested fallback semantics until a separate Rust-only accounting result contract exists. |
| Adapter/live account-state source parity | Deferred to RADP/RBTL/RTRACE owners. | PAPL-008 requires adapter/live source fixtures or traces and is outside the Rust core runtime task boundary. |

## Follow-Up Mapping

| Follow-up | Owner | Scope |
| --- | --- | --- |
| RCORE-014 | Rust Core Runtime Agent | Added Rust portfolio/accounting/PnL integration test; release golden trace remains RTRACE-owned. |
| RCORE-015 | Rust Core Runtime Agent | Close implementable portfolio/accounting/PnL gaps or record explicit deferrals. |
| RTRACE follow-up | Verification & Release Gatekeeper | Add executable `portfolio_pnl` golden trace replay and gate evidence. |
| RREM follow-up | Rust Core Runtime Agent + Gatekeeper | Remove Python/PyO3/Cython surfaces only after Rust usability and parity gates pass. |
