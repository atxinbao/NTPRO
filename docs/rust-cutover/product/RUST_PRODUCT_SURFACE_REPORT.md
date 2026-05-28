# Rust Product Surface Report

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-014

## Summary

NTPRO now has an owner-visible Rust product surface for the Rust-first cutover
phase. The current surface includes stable CLI help contracts, Rust example
smoke commands, Rust API documentation entrypoints, and explicit blockers for
workflow commands that still need runtime wiring.

This report does not claim that NTPRO is already a complete Rust-only product.
Python, PyO3, Cython, adapter parity, runtime parity, and release completion
remain governed by later runtime, adapter, QA, removal, and release tasks.

## Current Usable Surface

### CLI Help Contracts

The `nautilus-cli` package exposes help-level Rust-first contracts for the
product commands added during RPROD work:

```bash
cargo run -q -p nautilus-cli -- backtest --help
cargo run -q -p nautilus-cli -- sandbox --help
cargo run -q -p nautilus-cli -- live --help
cargo run -q -p nautilus-cli -- data --help
cargo run -q -p nautilus-cli -- config --help
```

The command contracts are recorded in:

- `docs/rust-cutover/product/RUST_CLI_CONTRACT.md`
- `docs/rust-cutover/product/BACKTEST_CLI_CONTRACT.md`
- `docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md`
- `docs/rust-cutover/product/DATA_CATALOG_CLI_CONTRACT.md`
- `docs/rust-cutover/product/CONFIG_VALIDATION_CLI_CONTRACT.md`

### Rust Backtest Example Smoke

The current backtest smoke is a Cargo example owned by `nautilus-backtest`:

```bash
cargo run -p nautilus-backtest --features examples --example engine-ema-cross
```

Recorded evidence:

- `docs/rust-cutover/evidence/RPROD-011.md`
- `examples/rust/backtest/README.md`

### Rust Sandbox Node Smoke

The current sandbox/live smoke is a Cargo example owned by `nautilus-live`:

```bash
cargo run -p nautilus-live --no-default-features --features node --example sandbox-node-smoke
```

Recorded evidence:

- `docs/rust-cutover/evidence/RPROD-012.md`
- `examples/rust/sandbox/README.md`

The smoke constructs a Rust `LiveNode` in `Sandbox` mode and does not connect
to a production venue.

### Rust API Documentation

Rust API documentation is generated without the PyO3 crate:

```bash
cargo doc --workspace --exclude nautilus-pyo3 --features arrow,ffi,high-precision,streaming,defi --no-deps
```

Primary doc roots:

- `target/doc/nautilus_cli/index.html`
- `target/doc/nautilus_backtest/index.html`
- `target/doc/nautilus_live/index.html`
- `target/doc/nautilus_data/index.html`
- `target/doc/nautilus_sandbox/index.html`
- `target/doc/nautilus_trading/index.html`

Recorded evidence:

- `docs/rust-cutover/product/RUST_API_ENTRYPOINTS.md`
- `docs/rust-cutover/evidence/RPROD-013.md`

## Evidence Map

| Task | Surface | Result |
| --- | --- | --- |
| `RPROD-001` | Rust CLI command contract | Product command contract recorded. |
| `RPROD-002` | Baseline CLI smoke | Existing CLI state and gaps recorded. |
| `RPROD-003` | Backtest CLI contract | Backtest command contract recorded. |
| `RPROD-004` | Backtest CLI help | Backtest help surface added. |
| `RPROD-005` | Live/sandbox contract | Live and sandbox command contract recorded. |
| `RPROD-006` | Live/sandbox CLI help | Live and sandbox help surfaces added. |
| `RPROD-007` | Data/catalog contract | Data/catalog command contract recorded. |
| `RPROD-008` | Data CLI help | Data help surface added. |
| `RPROD-009` | Config validation CLI | Config validation help surface added. |
| `RPROD-010` | Rust examples layout | Rust examples index recorded. |
| `RPROD-011` | Backtest example smoke | Cargo backtest smoke passed. |
| `RPROD-012` | Sandbox example smoke | Cargo sandbox node smoke passed. |
| `RPROD-013` | Rust API docs | Rust API entrypoint docs recorded. |
| `RPROD-014` | Product surface report | This report records surface status and blockers. |

## Current Gaps

The following gaps are owner-visible and remain in scope for later tasks:

- `nautilus backtest validate` and `nautilus backtest run` parse and expose help
  contracts, but full execution remains blocked until Rust config parsing,
  strategy selection, data catalog loading, and runtime wiring are completed.
- `nautilus sandbox validate`, `nautilus sandbox run`, `nautilus live validate`,
  and `nautilus live run` expose help contracts, but full execution remains
  blocked until live-node config parsing, lifecycle wiring, adapter
  classification, and sandbox/live smoke expansion are completed.
- `nautilus data inspect`, `nautilus data validate`, and `nautilus data load`
  expose help contracts, but data-source inspection and load behavior remain
  under later data/runtime and adapter work.
- `nautilus config validate` exposes the shared validation contract, but
  complete workflow-specific validation logic still needs Rust config models
  and parser wiring.
- Adapter support is not release-classified by this report. The `RADP-*` tasks
  must classify supported, deferred, or removed adapters with fixture, mock,
  schema, dry-run, or sandbox evidence.
- Runtime parity is not release-complete. The `RCORE-*` and `RBTL-*` tasks must
  produce lifecycle, backtest, live, data, execution, risk, portfolio, and
  model evidence.
- Golden trace and release QA are not complete. The `RTRACE-*` and `RREL-*`
  tasks must provide final regression, trace, release, and gate evidence.

## Scope Decisions

`SD-001` is the active removal gate:

- Python, PyO3, and Cython removal remains gated.
- No agent may delete or disable `python/**`, `nautilus_trader/**`,
  `crates/pyo3/**`, `build.py`, `pyproject.toml`, Cython files, or active
  product build paths until required gates are complete.
- Required gates include Rust product surface readiness, runtime smoke, adapter
  decisions, QA gate, release gatekeeper approval, and an explicit Rust-only
  route or removal task approval.

This report counts as product-surface evidence only. It does not approve
removal work and does not supersede `SD-001`.

## Recommended Next Work

1. Move from RPROD product surface into runtime and backtest/live tasks.
2. Classify adapter support under `RADP-*` before release or removal decisions.
3. Expand golden trace coverage before treating trading-semantic behavior as
   release-ready.
4. Keep Python, PyO3, and Cython deletion blocked until the release gatekeeper
   records the required removal evidence.

## Verification

The required RPROD-014 verification command is:

```bash
scripts/ai/verify_full.sh
```

Task-level command results are recorded in:

- `docs/rust-cutover/evidence/RPROD-014.md`
