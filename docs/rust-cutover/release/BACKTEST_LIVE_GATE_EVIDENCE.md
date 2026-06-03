# Backtest/Live Gate Evidence

Date: 2026-05-30
Executor: Codex
Task ID: RBTL-010

## Gate Status

The R4 backtest/live gate is usable and scoped, but it is not a final
Rust-only release signoff. Current evidence proves that Rust can run scoped
backtest and live/sandbox paths without Python, and that the covered
backtest/live traces have executable Rust evidence.

As of 2026-06-03, broader Rust-only completion approval is recorded by RREL-008
after RREL-009 passed final local release verification. This document remains
the scoped backtest/live gate record and does not create a release tag or
publish a GitHub Release.

## Standard Command

Run the full gate evidence with:

```bash
scripts/ai/verify_full.sh
```

For the current local toolchain layout, the passing command was run with the
pinned Rust 1.95.0 toolchain:

```bash
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc scripts/ai/verify_full.sh
```

## Current Executable Evidence

| Area | Evidence | Rust entrypoint | Covered behavior |
| --- | --- | --- | --- |
| Backtest direct engine | RBTL-001, RTRACE-005, RBTL-009 | `nautilus-backtest::BacktestEngine` tests and golden trace harness | Deterministic single quote replay, normalized `BacktestResult`, no orders or positions in scoped fixture. |
| Backtest node/catalog | RBTL-002, RBTL-003 | `nautilus-backtest::BacktestNode` integration tests | Deterministic Parquet catalog smoke, explicit local `file` protocol, missing requested instrument rejection. |
| Backtest result/report | RBTL-004 | `nautilus-backtest` integration tests | Result report fields are present for scoped PnL, PnL%, and Long Ratio evidence. |
| Live node lifecycle | RBTL-005, RBTL-006 | `nautilus-live::LiveNode` builder and integration tests | Rust `LiveNode` can build, start, enter `Running`, stop, and enter `Stopped` in sandbox mode. |
| Live config/client boundary | RBTL-007 | `nautilus-live` config and node tests | Unsupported config client maps fail fast; explicit Rust data/execution client factory registration works. |
| Sandbox execution smoke | RBTL-008 | `nautilus-live` node test with `nautilus-sandbox` factory | Rust sandbox execution client registers, starts, writes starting account state, and stops without Python. |
| Backtest/live semantic parity | RBTL-009 | `nautilus-backtest::backtest_live_semantic_parity` | Backtest result and live sandbox lifecycle are normalized into one scoped semantic parity golden trace. |

## Current Trace Inventory

| File | Rows | Category | Execution status |
| --- | ---: | --- | --- |
| `tests/golden/backtest_replay_schema.jsonl` | 1 | `backtest_live` | Rust backtest replay harness |
| `tests/golden/live_sandbox_lifecycle_schema.jsonl` | 1 | `backtest_live` | Rust live/sandbox lifecycle harness |
| `tests/golden/backtest_live_semantic_parity_schema.jsonl` | 1 | `backtest_live` | Rust backtest/live semantic parity harness |

The broader golden trace inventory is recorded in
`docs/rust-cutover/golden_trace/GATE_EVIDENCE.md`.

## What Is Green For This Gate

- Rust `BacktestEngine` can run a deterministic quote replay and return a
  normalized result without Python.
- Rust `BacktestNode` can stream deterministic catalog data through the node
  path and preserve inspectable engine state for tests.
- Rust backtest catalog boundaries now have explicit local protocol and
  missing-instrument coverage.
- Rust `LiveNode` can complete a sandbox start/stop lifecycle without Python.
- Rust live builder registration supports explicit data and execution client
  factories.
- Rust sandbox execution client smoke verifies startup account state is written
  into cache and then cleanly stops.
- A scoped parity trace now ties the backtest and live/sandbox evidence into one
  executable backtest/live semantic parity check.

## Scoped Follow-Ups

The gate is scoped rather than final because these gaps remain:

- Rust CLI workflow execution is still not the final product path for backtest,
  sandbox, and live runs. Help/contracts exist, but runtime config-to-run wiring
  remains owned by product-surface follow-up tasks.
- Backtest strategy/actor/execution-algorithm wiring is still largely manual in
  Rust examples and tests. A stable user config to Rust strategy registry is not
  complete.
- `BacktestNode` still has known multi-run/kernel-message-bus limitations that
  must stay scoped until later runtime work closes or explicitly defers them.
- Rust live config maps for `data_clients` and `exec_clients` are intentionally
  rejected until a Rust adapter factory registry or per-adapter builder contract
  exists.
- Some live runtime config fields remain explicitly rejected until their
  database/cache/event-store behavior is wired or removed from the supported
  Rust product surface.
- Live startup still has a scoped cancellation gap around long-running client
  connection futures during early startup.
- Broader Rust-only release completion is tracked by RREL-008/RREL-009 rather
  than this scoped RBTL document.

## Removal Gate Impact

This document does not authorize deleting Python, PyO3, Cython, `build.py`,
`pyproject.toml`, or legacy Python package paths. It records that the
backtest/live gate has executable Rust evidence and explicit remaining blockers.

Removal and completion approval are tracked by the dedicated RREM/RREL task
evidence. This document remains scoped to backtest/live evidence only.
