# RREM-005 Python Test Scope Map

Date: 2026-06-01
Executor: Codex
Task ID: RREM-005

Updated: 2026-06-04
Executor: Codex
Follow-up ID: RC-CLEANUP-001

## Scope

This document classifies Python-only tests for Rust-only removal planning. It
does not delete, rewrite, skip, or weaken any Python, PyO3, Cython, Rust, or
golden trace tests.

## RC Cleanup Decision

The original RREM-005 scope map was intentionally non-destructive. After the
Rust-only completion gate passed, `ntpro-rust-only-rc.1` was created, and the
human owner approved public release cleanup, the top-level legacy Python tests
under `tests/**/*.py` were removed from the release surface. That cleanup is
included in the current `ntpro-rust-only-rc.2` source point.

The deletion decision is:

```text
removed_with_surface
```

Reason:

- The Python/PyO3/Cython product surfaces are no longer supported release
  surfaces.
- RREL-009 made the final local Rust-only release verification green.
- RREL-008 recorded owner-approved Rust-only completion.
- `ntpro-rust-only-rc.2` exists as the current tag-only release candidate.
- The legacy top-level Python tests were still the dominant GitHub language
  signal and no longer represented the public product direction.

This RC cleanup keeps local Python control/verification scripts under
`scripts/`. Those scripts are repository automation, not a product API.

## Summary

The table below is the original RREM-005 inventory snapshot. It is retained as
historical removal evidence. The current repository no longer tracks the
top-level Python files under `tests/**/*.py`.

| Test class | Count | Current Rust evidence | Scope decision |
| --- | ---: | --- | --- |
| Top-level Python tests under `tests/**` | 534 files at RREM-005; 0 tracked Python files after RC cleanup | 156 Rust crate test files, golden trace tests, adapter fixture manifests | Removed with the retired Python/PyO3/Cython product surface after release-gate approval. |
| Python package overlay tests under `python/tests/**` | 90 files | Limited Rust replacement; mostly package/stub/acceptance coverage | Defer until Python package surface is removed or replaced. |
| Tests mentioning PyO3/Cython/legacy interop | 179 files | RREM-002/RREM-003 inventories and some Rust parity tests | Port or explicitly retire with migration notes before removal. |
| Adapter integration Python tests | 185 files under `tests/integration_tests/adapters` | 92 adapter Rust test files plus RADP fixture manifests | Port/defer by venue; fixture-backed Rust coverage does not imply all Python adapter tests are removable. |
| Python unit tests | 284 files under `tests/unit_tests` | Rust tests across model/backtest/live/execution/risk/portfolio/etc. | Split by domain; remove only after matching Rust crate evidence. |
| Performance tests | 24 files | Some Rust benchmarks/tests, but no one-to-one replacement map | Defer unless a Rust perf baseline exists. |
| Memory leak tests | 18 files | No Rust-equivalent leak harness found in scope | Defer or replace with Rust/tooling-specific leak evidence. |
| Acceptance/docs tests | 5 files | CLI help/docs evidence exists, but product run paths remain blocked | Defer until Rust CLI run and docs migration are complete. |

## Original Test Inventory Snapshot

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
| `tests/unit_tests/model/**` | Removed with surface in RC cleanup. | The Python/PyO3 product surface these tests targeted is no longer supported. Rust model tests, golden traces, and release gates remain the active evidence path. |
| `tests/unit_tests/backtest/**` | Removed with surface in RC cleanup. | Rust backtest tests, CLI help contracts, and release verification remain the active evidence path. |
| `tests/unit_tests/live/**` | Removed with surface in RC cleanup. | Rust live/sandbox evidence and release verification remain the active evidence path. |
| `tests/unit_tests/execution/**` | Removed with surface in RC cleanup. | Rust execution tests and golden trace evidence remain the active evidence path. |
| `tests/unit_tests/risk/**` | Removed with surface in RC cleanup. | Rust risk tests and release verification remain the active evidence path. |
| `tests/unit_tests/portfolio/**` and `tests/unit_tests/accounting/**` | Removed with surface in RC cleanup. | Rust portfolio/accounting tests and release evidence remain the active evidence path. |
| `tests/unit_tests/persistence/**` | Removed with surface in RC cleanup. | Rust persistence tests and release verification remain the active evidence path. |
| `tests/unit_tests/indicators/**` and `tests/unit_tests/analysis/**` | Removed with surface in RC cleanup. | Rust crate tests and Rust-only documentation remain the active evidence path. |
| `tests/integration_tests/adapters/**` | Removed with surface in RC cleanup. | RADP fixture manifests and adapter Rust tests remain the active evidence path. |
| `tests/performance_tests/**` | Removed with surface in RC cleanup. | Python performance tests targeted the retired product surface; future performance evidence must be Rust-native. |
| `tests/mem_leak_tests/**` | Removed with surface in RC cleanup. | Python/Cython/PyO3 leak checks do not represent the Rust-only product surface. |
| `tests/acceptance_tests/**` | Removed with surface in RC cleanup. | Rust CLI and release verification remain the active acceptance path. |
| `tests/docs_tests/**` | Removed with surface in RC cleanup. | README and cutover docs are now Rust-only public surfaces. |
| `python/tests/**` | Defer until Python package overlay removal decision. | These tests validate the Python package/stubs/acceptance surface itself. |

## Deletion Preconditions

Before any future Python test file is added or restored, a later RREM/RREL task
must record why that test belongs in a Rust-only repository. Historical Python
test removal used one of these outcomes:

- `ported`: a Rust test or golden trace covers the same behavior;
- `superseded`: a Rust product contract intentionally replaces the workflow;
- `deferred`: the workflow remains out of Rust-only release scope with
  release-gate approval;
- `legacy_migration_only`: the test belongs to a retained migration archive,
  not the product test gate;
- `removed_with_surface`: the underlying Python/PyO3/Cython product surface is
  removed with migration notes and release approval.

Do not restore Python tests as product evidence for the Rust-only release. New
test coverage should be Rust-native unless the file is a local repository
automation script and is explicitly documented as non-product.

## Immediate Follow-Ups

- Keep any future regression coverage Rust-native.
- Keep local Python scripts limited to repository control and release evidence.
- Do not reintroduce Python/PyO3/Cython product tests without a new owner
  decision and release-gate record.
