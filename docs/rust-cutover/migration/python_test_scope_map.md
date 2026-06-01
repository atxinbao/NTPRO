# RREM-005 Python Test Scope Map

Date: 2026-06-01
Executor: Codex
Task ID: RREM-005

## Scope

This document classifies Python-only tests for Rust-only removal planning. It
does not delete, rewrite, skip, or weaken any Python, PyO3, Cython, Rust, or
golden trace tests.

## Summary

Python tests remain a major parity and migration surface. Some areas already
have Rust crate tests and golden trace evidence, but many Python tests still
cover PyO3/Cython interop, Python package imports, adapter integration
behavior, memory leaks, performance, and docs/tutorial behavior that cannot be
deleted safely until replacement evidence is explicit.

| Test class | Count | Current Rust evidence | Scope decision |
| --- | ---: | --- | --- |
| Top-level Python tests under `tests/**` | 534 files | 156 Rust crate test files, golden trace tests, adapter fixture manifests | Scope by family; do not bulk-delete. |
| Python package overlay tests under `python/tests/**` | 90 files | Limited Rust replacement; mostly package/stub/acceptance coverage | Defer until Python package surface is removed or replaced. |
| Tests mentioning PyO3/Cython/legacy interop | 179 files | RREM-002/RREM-003 inventories and some Rust parity tests | Port or explicitly retire with migration notes before removal. |
| Adapter integration Python tests | 185 files under `tests/integration_tests/adapters` | 92 adapter Rust test files plus RADP fixture manifests | Port/defer by venue; fixture-backed Rust coverage does not imply all Python adapter tests are removable. |
| Python unit tests | 284 files under `tests/unit_tests` | Rust tests across model/backtest/live/execution/risk/portfolio/etc. | Split by domain; remove only after matching Rust crate evidence. |
| Performance tests | 24 files | Some Rust benchmarks/tests, but no one-to-one replacement map | Defer unless a Rust perf baseline exists. |
| Memory leak tests | 18 files | No Rust-equivalent leak harness found in scope | Defer or replace with Rust/tooling-specific leak evidence. |
| Acceptance/docs tests | 5 files | CLI help/docs evidence exists, but product run paths remain blocked | Defer until Rust CLI run and docs migration are complete. |

## Current Test Inventory

Python tests under `tests/**`:

| Family | Count |
| --- | ---: |
| `tests/unit_tests` | 284 |
| `tests/integration_tests` | 201 |
| `tests/performance_tests` | 24 |
| `tests/mem_leak_tests` | 18 |
| `tests/acceptance_tests` | 3 |
| `tests/docs_tests` | 2 |
| root pytest support files | 2 |

Python package overlay tests under `python/tests/**`:

| Family | Count |
| --- | ---: |
| `python/tests/unit` | 79 |
| `python/tests/strategies` | 3 |
| `python/tests/acceptance` | 3 |
| root/support files | 5 |

Largest domain groupings under `tests/**`:

| Domain grouping | Count |
| --- | ---: |
| `tests/integration_tests/adapters` | 185 |
| `tests/unit_tests/model` | 64 |
| `tests/unit_tests/indicators` | 39 |
| `tests/unit_tests/backtest` | 29 |
| `tests/unit_tests/analysis` | 23 |
| `tests/unit_tests/common` | 21 |
| `tests/unit_tests/persistence` | 13 |
| `tests/unit_tests/core` | 12 |
| `tests/unit_tests/live` | 12 |
| `tests/unit_tests/execution` | 11 |
| `tests/unit_tests/portfolio` | 9 |
| `tests/unit_tests/accounting` | 7 |
| `tests/unit_tests/data` | 7 |
| `tests/unit_tests/trading` | 6 |
| `tests/unit_tests/risk` | 5 |

## Rust Test Evidence Inventory

Rust test and example coverage exists, but it is not a blanket replacement for
all Python tests:

| Rust evidence family | Count / examples |
| --- | --- |
| Rust crate test files under `crates/**/tests/*.rs` | 156 |
| Rust crate examples under `crates/**/examples/*.rs` | 84 |
| Rust files mentioning golden trace/parity/fixture terms | 97 |
| Adapter Rust tests | 92 files across venue crates |
| Backtest Rust tests | 10 files, including golden trace and semantic parity |
| Live Rust tests | 6 files, including sandbox golden trace |
| Event store Rust tests | 8 files |
| Plugin Rust tests | 8 files |
| Network Rust tests | 6 files |

Important release evidence already recorded:

- `docs/rust-cutover/golden_trace/GATE_EVIDENCE.md`
- `docs/rust-cutover/release/BACKTEST_LIVE_GATE_EVIDENCE.md`
- `docs/rust-cutover/inventory/*_adapter_gaps.md`
- `docs/rust-cutover/migration/python_to_rust_workflow_map.md`

## Port / Scope Decisions

| Python test family | Decision | Why |
| --- | --- | --- |
| `tests/unit_tests/model/**` | Port/replace by Rust model tests before removal. | Model value/object/order/instrument semantics are public runtime contracts. Existing Rust value gates are not enough to remove every Python model test. |
| `tests/unit_tests/backtest/**` | Partially covered; port remaining backtest node/engine/config behavior. | Rust backtest tests and golden trace exist, but CLI/config workflow remains blocked. |
| `tests/unit_tests/live/**` | Partially covered; port lifecycle/config behavior that is not in Rust tests. | Rust live tests cover node lifecycle, but Python execution client/reconciliation tests remain migration blockers. |
| `tests/unit_tests/execution/**` | Port or explicitly scope before removal. | Execution semantics affect order lifecycle and trading behavior; release trace is not complete for every Python scenario. |
| `tests/unit_tests/risk/**` | Port or explicitly scope before removal. | PyO3 Greeks and risk behavior need Rust-native replacement or decision. |
| `tests/unit_tests/portfolio/**` and `tests/unit_tests/accounting/**` | Port/replace before removal. | Portfolio/accounting/PnL remain release-gate-sensitive. |
| `tests/unit_tests/persistence/**` | Partially covered; port catalog/wrangler/streaming gaps. | Rust persistence tests exist, but Python/PyO3 catalog and wrangler tests remain. |
| `tests/unit_tests/indicators/**` and `tests/unit_tests/analysis/**` | Port or defer with explicit scope. | Indicator/analysis behavior may be public strategy input; Rust replacement needs coverage map. |
| `tests/integration_tests/adapters/**` | Scope by venue using RADP fixture manifests. | Adapter Rust tests are broad but not one-to-one with Python integration behavior. |
| `tests/performance_tests/**` | Defer until Rust benchmark/perf baseline exists. | Performance expectations are not interchangeable with functional tests. |
| `tests/mem_leak_tests/**` | Defer or replace with Rust-specific memory/leak checks. | Current tests exercise Python/Cython/PyO3 memory behavior and cannot prove Rust leak safety directly. |
| `tests/acceptance_tests/**` | Defer until Rust CLI product run paths pass. | CLI help exists, but product run replacement remains blocked. |
| `tests/docs_tests/**` | Defer until docs are Rust-only and Python docs are migrated. | Current docs still include Python tutorial content. |
| `python/tests/**` | Defer until Python package overlay removal decision. | These tests validate the Python package/stubs/acceptance surface itself. |

## Deletion Preconditions

Before a Python test file can be removed, a later RREM/RREL task must record at
least one of these outcomes:

- `ported`: a Rust test or golden trace covers the same behavior;
- `superseded`: a Rust product contract intentionally replaces the workflow;
- `deferred`: the workflow remains out of Rust-only release scope with
  release-gate approval;
- `legacy_migration_only`: the test belongs to a retained migration archive,
  not the product test gate;
- `removed_with_surface`: the underlying Python/PyO3/Cython product surface is
  removed with migration notes and release approval.

Do not mark a Python test removable only because a same-named Rust crate test
exists. The behavior, fixture, data shape, error path, and public contract must
match or be explicitly scoped out.

## Immediate Follow-Ups

- Build a per-domain port checklist for model, backtest, live, execution,
  risk, portfolio/accounting, persistence, indicators, and adapters.
- Link adapter Python integration tests to RADP fixture manifests by venue.
- Link Python acceptance/docs tests to the Rust CLI product readiness matrix.
- Keep PyO3/Cython interop tests until the PyO3/Cython product surfaces are
  removed or replaced by Rust-native tests.
