# Rust CLI Command Contract

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-001

## Purpose

This contract defines the Rust-first command-line surface NTPRO must expose
before Python, PyO3, or Cython removal can advance. It is a product contract,
not an implementation patch. Later RPROD tasks may implement, refine, or record
blockers against this contract, but they must keep user-visible behavior
traceable here.

## Current CLI Baseline

The current binary is `nautilus`, provided by the `nautilus-cli` package.

Observed default command surface:

```text
nautilus
  database
```

Observed feature-gated command surface:

```text
nautilus --features defi
  database
  blockchain
```

Current product-surface blockers:

- `nautilus backtest` is not implemented.
- `nautilus sandbox` is not implemented.
- `nautilus live` is not implemented.
- Rust docs and examples exist, but they are not yet tied to a stable Rust CLI
  command contract.

## Global CLI Rules

All Rust product commands must follow these rules:

- The binary name is `nautilus`.
- Commands must run without importing Python.
- Commands must not require `crates/pyo3`, Cython, maturin, or generated Python
  extension artifacts.
- `--help` must exit successfully for every supported command and subcommand.
- Invalid input must exit non-zero and print a concise error.
- Run commands must accept a config file path.
- Validate commands must parse and validate the same config file shape used by
  the corresponding run command.
- Every command that touches external adapters must support fixture, mock,
  schema, dry-run, or sandbox validation before it is used as release evidence.

## Config Contract

Rust CLI config is the public user input boundary for Rust-first workflows.

Required config behavior:

- Config files must be parsed by Rust code into Rust config structs.
- Config parsing must not import Python or call Python package code.
- Config validation must report the failing section and field where possible.
- Run commands must reject a config that the matching validate command rejects.
- Adapter credentials and venue-specific connection details must be provided
  through explicit config fields or environment variables documented by the
  adapter task.
- Config examples must be committed with the docs or examples task that first
  makes a workflow supported.

Initial supported config groups:

- `system`: trader ID, instance ID, environment, logging.
- `data`: catalog path, data type, instrument ID, time range, input source.
- `venue`: venue name, account type, OMS type, starting balances, routing.
- `strategy`: strategy name, parameters, subscribed instruments.
- `execution`: run mode, reconciliation behavior, shutdown behavior.
- `output`: result directory, report format, trace output.

The exact file format can be finalized by the implementation task. Until then,
new Rust CLI work must document whether it accepts TOML, JSON, or another
format, and must include a validation command for that format.

## Required Commands

### `nautilus backtest`

Purpose: run deterministic historical simulations through Rust runtime code.

Required subcommands:

```text
nautilus backtest validate --config <path>
nautilus backtest run --config <path> [--run-id <id>] [--output <dir>]
```

Minimum contract:

- `validate` loads the config, validates data source, venue, and strategy
  sections, then exits without running the engine.
- `run` uses `nautilus-backtest` Rust APIs such as `BacktestEngine` or
  `BacktestNode`.
- `run` must produce an owner-visible run ID.
- `run` must write or print enough result metadata for later golden trace and
  parity tasks to compare behavior.
- Unsupported inputs must be rejected with an explicit blocker, not silently
  ignored.

Evidence needed before this command is considered usable:

- `nautilus backtest --help` passes.
- `nautilus backtest validate --help` passes.
- `nautilus backtest run --help` passes.
- At least one deterministic Rust backtest smoke passes without Python.

### `nautilus sandbox`

Purpose: run paper/sandbox execution through Rust live-node and sandbox adapter
paths without sending real orders to an external venue.

Required subcommands:

```text
nautilus sandbox validate --config <path>
nautilus sandbox run --config <path> [--run-id <id>] [--output <dir>]
```

Minimum contract:

- `validate` checks live-node and sandbox adapter config without starting the
  node.
- `run` uses Rust live-node construction with sandbox execution components.
- `run` must expose startup and shutdown outcomes.
- The first supported sandbox path may use fixtures or local deterministic
  market data.

Evidence needed before this command is considered usable:

- `nautilus sandbox --help` passes.
- `nautilus sandbox validate --help` passes.
- `nautilus sandbox run --help` passes.
- At least one Rust sandbox lifecycle smoke passes without Python.

### `nautilus live`

Purpose: run Rust live-node workflows for scoped adapters.

Required subcommands:

```text
nautilus live validate --config <path>
nautilus live run --config <path> [--run-id <id>] [--output <dir>]
```

Minimum contract:

- `validate` checks the live-node config and adapter config without connecting
  to a production venue unless a later adapter task explicitly scopes that
  behavior.
- `run` uses `nautilus-live` Rust APIs such as `LiveNode` and `LiveNodeBuilder`.
- `run` must expose startup, reconciliation, stop, and shutdown outcomes.
- Adapter-specific runtime support must be classified as supported, deferred,
  or blocked in adapter evidence.

Evidence needed before this command is considered usable:

- `nautilus live --help` passes.
- `nautilus live validate --help` passes.
- `nautilus live run --help` passes.
- At least one Rust live or sandbox node lifecycle smoke passes without Python.

### `nautilus data`

Purpose: inspect and validate catalog/data-provider inputs for backtest and
runtime workflows.

Required subcommands:

```text
nautilus data inspect --config <path>
nautilus data validate --config <path>
```

Minimum contract:

- `inspect` reports catalog/data source metadata without running a strategy.
- `validate` checks data availability, instrument mapping, and requested time
  range.
- Data-provider adapter behavior remains under RADP tasks.

### `nautilus database`

Purpose: keep existing database administration support.

Current supported subcommands:

```text
nautilus database init
nautilus database drop
```

RPROD tasks must not break these commands while adding Rust product commands.

### `nautilus blockchain`

Purpose: keep current DeFi/blockchain operational commands behind the existing
`defi` feature.

Current feature-gated subcommands:

```text
nautilus blockchain sync-blocks
nautilus blockchain sync-dex
nautilus blockchain analyze-pool
```

These commands are not blockers for backtest, sandbox, or live Rust-first
product surface unless a later adapter task explicitly scopes them.

## Output Contract

Run commands must expose:

- command name;
- run ID;
- config path;
- start time and completion status;
- output directory or artifact paths when configured;
- blocker or error details on failure.

Human-readable output is sufficient for early RPROD tasks. Machine-readable
output can be added later, but it must not replace human-readable errors.

## Gate Mapping

- RPROD-002 proves current CLI help/version smoke or records gaps.
- RPROD-003 and RPROD-004 cover the backtest command.
- RPROD-005 and RPROD-006 cover live/sandbox commands.
- RPROD-007 through RPROD-009 cover data/catalog and config validation.
- RPROD-010 through RPROD-014 cover examples, docs, and final product-surface
  report.
- RREM tasks must not delete Python/PyO3/Cython product surfaces until the
  required Rust CLI/API/example usability evidence is complete.
