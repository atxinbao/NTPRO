# Rust Portfolio / Accounting / PnL Gap Inventory

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-013

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
| PAPL-003 | Open: no Rust-only end-to-end replay from order fill through account balance, realized PnL, unrealized PnL, equity, and `PortfolioSnapshot`. | Unit tests exercise pieces in `crates/portfolio/tests/portfolio.rs` and `crates/portfolio/src/manager.rs`, but the golden trace gate still has no `portfolio_pnl` replay. | RCORE/RTRACE follow-up must bind a deterministic fill/position/account stream to portfolio outputs. |
| PAPL-004 | Partial: account-state calculation flags are not implemented on Rust account trait implementations. | `CashAccount::calculated_account_state` and `MarginAccount::calculated_account_state` currently return `false` with TODO comments. Generated `AccountState` paths exist in `AccountsManager`, but the trait flag does not express calculated-state readiness. | Requires a later RCORE decision: implement the flag semantics with tests or record that the flag is not part of the Rust-only product contract. |
| PAPL-005 | Partial: missing exchange-rate and missing-balance paths degrade through logs, `None`, or zero fallback rather than a unified typed accounting result. | `AccountsManager::update_balance_single_currency` returns early on missing xrate or balance; `update_balance_multi_currency` rejects missing negative debit currencies; `Portfolio::calculate_realized_pnl` can return zero when xrate is unavailable and marks pending calculations. Existing tests cover several of these paths. | Later runtime tasks need explicit acceptance criteria for whether this remains Python-parity behavior or becomes a Rust-only typed error contract. |
| PAPL-006 | Partial: snapshot/equity is available in Rust but not release replay evidence. | `Portfolio::build_snapshot`, `Portfolio::snapshots`, snapshot timers, and `PortfolioSnapshot` exist and have tests. The trace gate still lacks account balance, margin, realized PnL, unrealized PnL, and equity replay. | Release gate needs deterministic snapshot output comparison, not only unit tests. |
| PAPL-007 | Partial: multi-account and multi-venue aggregation has targeted Rust tests but no gate trace. | `crates/portfolio/tests/portfolio.rs` covers account-id filters for net exposure, unrealized PnL, realized PnL, and multiple account/equity cases. | Needs release-level trace or explicit scope decision for supported multi-account/multi-venue workflows. |
| PAPL-008 | Open: adapter/live account-state sources are not proven through portfolio PnL replay. | Adapter tests and Python integration tests include account/balance payloads, but RCORE-013 did not find a Rust `portfolio_pnl` trace that starts from adapter or live account-state events and verifies portfolio outputs. | Adapter/live account-state parity remains owned by later RBTL/RADP/RTRACE tasks. |

## Non-Goals Preserved

- No Python, PyO3, Cython, `build.py`, or `pyproject.toml` removal.
- No public API changes.
- No trading-semantic changes.
- No changes to account, margin, PnL, equity, snapshot, adapter, or persistence
  behavior.

## Follow-Up Mapping

| Follow-up | Owner | Scope |
| --- | --- | --- |
| RCORE-014 | Rust Core Runtime Agent | Add or identify Rust tests for portfolio/accounting/PnL gaps. |
| RCORE-015 | Rust Core Runtime Agent | Close implementable portfolio/accounting/PnL gaps or record explicit deferrals. |
| RTRACE follow-up | Verification & Release Gatekeeper | Add executable `portfolio_pnl` golden trace replay and gate evidence. |
| RREM follow-up | Rust Core Runtime Agent + Gatekeeper | Remove Python/PyO3/Cython surfaces only after Rust usability and parity gates pass. |
