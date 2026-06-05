# Backtest CLI Run Contract

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-003

Updated: 2026-06-06
Executor: Codex
Task ID: DRG-005

## Purpose

This contract refines the `nautilus backtest` surface from
`docs/rust-cutover/product/RUST_CLI_CONTRACT.md`. It defines the first Rust-first
backtest run command shape, config boundary, output contract, and failure
behavior for later implementation tasks.

## Current Baseline

The current `nautilus-cli` binary exposes `backtest` by default:

```text
nautilus backtest --help
```

RHARD-006 adds a metadata-only minimal path:

```text
nautilus backtest validate --config examples/rust/backtest/minimal_dry_run.toml
nautilus backtest run --config examples/rust/backtest/minimal_dry_run.toml --dry-run --output runs/minimal-backtest-dry-run
```

This path validates the config and writes a summary file, but it does not start
`BacktestEngine`, load market data, or run a trading strategy.

DRG-005 adds the first scoped non-dry-run CLI path:

```text
nautilus backtest validate --config examples/rust/backtest/minimal_engine_smoke.toml
nautilus backtest run --config examples/rust/backtest/minimal_engine_smoke.toml --output runs/minimal-backtest-engine-smoke
```

This starts the Rust `BacktestEngine` with synthetic AUD/USD quotes and the
Rust `EmaCross` strategy. It is intentionally limited to
`run.mode = "engine-smoke"`, `strategy.name = "ema-cross"`, and
`data.instrument_id = "AUD/USD.SIM"`.

## Command Surface

The Rust-first backtest command must expose:

```text
nautilus backtest validate --config <path>
nautilus backtest run --config <path> [--run-id <id>] [--output <dir>] [--dry-run]
```

### `validate`

`validate` must parse the same config shape accepted by `run`, validate the
configuration, then exit without building or running a backtest engine.

Validation must check:

- config file exists and parses;
- exactly one top-level backtest run is selected for the first supported CLI
  version;
- at least one venue is configured;
- at least one data source is configured;
- venue, data, and run time fields can be converted into Rust config structs;
- `start` is earlier than `end` when both are provided;
- `run.id` or `--run-id` is non-empty;
- output settings are syntactically valid.

`validate` must not import Python, call Python package code, require PyO3, or
require Cython build artifacts.

### `run`

`run --dry-run` performs the RHARD-006 metadata-only path and does not start the
engine. `run` without `--dry-run` performs the DRG-005 engine-smoke path. It
must perform the same validation as `validate`, then execute only the scoped
Rust backtest APIs documented below.

Allowed Rust integration points:

- `nautilus_backtest::config::BacktestRunConfig`;
- `nautilus_backtest::config::BacktestDataConfig`;
- `nautilus_backtest::config::BacktestVenueConfig`;
- `nautilus_backtest::node::BacktestNode`;
- `nautilus_backtest::engine::BacktestEngine` only for explicitly scoped
  low-level backtest work.

The command must not silently fall back to Python backtesting.

## Config Format

The first CLI implementation should use TOML because it is readable for command
line users and maps cleanly to nested Rust structs. JSON may be added later, but
the initial supported format must be explicit in `--help`.

Minimum TOML shape:

```toml
[run]
id = "ema-cross-run"
start = "2025-01-01T00:00:00Z"
end = "2025-01-02T00:00:00Z"
chunk_size = 100
raise_exception = true
dispose_on_completion = true

[[venues]]
name = "SIM"
oms_type = "HEDGING"
account_type = "MARGIN"
book_type = "L1_MBP"
starting_balances = ["1000000 USD"]
routing = false

[[data]]
data_type = "QuoteTick"
catalog_path = "catalog/backtests/ema-cross"
instrument_id = "AUD/USD.SIM"
start_time = "2025-01-01T00:00:00Z"
end_time = "2025-01-02T00:00:00Z"

[strategy]
type = "EmaCross"
instrument_id = "AUD/USD.SIM"
trade_size = "100000"
fast_period = 10
slow_period = 20

[output]
dir = "runs/ema-cross-run"
format = "text"
write_summary = true
```

Current RHARD-006 minimal TOML shape:

```toml
[run]
id = "minimal-backtest-dry-run"
mode = "dry-run"

[data]
source = "synthetic-quotes"
instrument_id = "AUD/USD.SIM"
quotes = 3

[strategy]
name = "no-op"

[output]
dir = "runs/minimal-backtest-dry-run"
```

Current DRG-005 engine-smoke TOML shape:

```toml
[run]
id = "minimal-backtest-engine-smoke"
mode = "engine-smoke"

[data]
source = "synthetic-quotes"
instrument_id = "AUD/USD.SIM"
quotes = 120

[strategy]
name = "ema-cross"
trade_size = "100000"
fast_period = 10
slow_period = 20

[output]
dir = "runs/minimal-backtest-engine-smoke"
```

### Field Mapping

`run` maps to `BacktestRunConfig`:

- `id` maps to `BacktestRunConfig::id`.
- `chunk_size` maps to streaming chunk size.
- `raise_exception` maps to exception behavior.
- `dispose_on_completion` maps to engine disposal behavior.
- `start` and `end` map to UTC run boundaries.

`venues` maps to `BacktestVenueConfig`:

- `name`;
- `oms_type`;
- `account_type`;
- `book_type`;
- `starting_balances`;
- routing and simulation options as supported by the Rust config type.

`data` maps to `BacktestDataConfig`:

- `data_type`;
- `catalog_path`;
- optional filesystem protocol and storage options;
- `instrument_id` or `instrument_ids`;
- `start_time` and `end_time`;
- `filter_expr`;
- `client_id`;
- `bar_spec` or explicit `bar_types`;
- `optimize_file_loading`.

`strategy` is the implementation blocker for the first CLI run path. The CLI
needs a Rust-native strategy registry or an explicitly scoped strategy example
contract before arbitrary user strategies can be loaded from config. Until that
exists, `run` may support a named built-in/example strategy only if the command
prints that support boundary clearly.

`output` controls owner-visible artifacts and must not affect trading semantics.

## Output Contract

`validate` must print a concise success or failure summary:

```text
backtest.validate status=ok config=<path> run_id=<id>
```

`run --dry-run` must print and write:

- command name;
- run ID;
- config path;
- input source;
- instrument ID;
- quote count;
- strategy name;
- output directory;
- summary path;
- `engine_started=false`;
- `runtime_status=deferred`.

DRG-005 engine-smoke `run` must print or write:

- command name;
- run ID;
- config path;
- output directory;
- input source;
- instrument ID;
- quote count;
- strategy name;
- `engine_started=true`;
- `runtime_status=completed`.

Human-readable text is enough for the initial implementation. Machine-readable
JSON output can be added later as an explicit `--format json` option.

## Failure Behavior

The command must use stable non-zero exits so automation can distinguish user
errors from runtime failures.

Recommended exit codes:

- `2`: CLI usage or argument parse error;
- `3`: config parse or validation error;
- `4`: data catalog or data query unavailable;
- `5`: backtest build error;
- `6`: backtest runtime error;
- `7`: output artifact write error.

Every failure must name the failing section or operation when known. Unsupported
config sections must be rejected explicitly instead of ignored.

## Implementation Gates

The command is not considered usable until all of the following pass:

```bash
cargo run -q -p nautilus-cli -- backtest --help
cargo run -q -p nautilus-cli -- backtest validate --help
cargo run -q -p nautilus-cli -- backtest run --help
cargo run -q -p nautilus-cli -- backtest validate --config examples/rust/backtest/minimal_dry_run.toml
cargo run -q -p nautilus-cli -- backtest run --config examples/rust/backtest/minimal_dry_run.toml --dry-run --output runs/minimal-backtest-dry-run
cargo run -q -p nautilus-cli -- backtest validate --config examples/rust/backtest/minimal_engine_smoke.toml
cargo run -q -p nautilus-cli -- backtest run --config examples/rust/backtest/minimal_engine_smoke.toml --output runs/minimal-backtest-engine-smoke
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
```

The first successful run smoke must also prove:

- no Python import is required;
- no PyO3 or Cython build artifact is required;
- a deterministic Rust backtest path runs from config;
- the run emits an owner-visible run ID and status.

## Remaining Blockers

- Arbitrary `nautilus backtest run` strategy loading from config has no stable
  Rust product contract yet.
- Adapter/catalog-backed market data loading is not implemented by this CLI
  path.
- The DRG-005 engine-smoke path supports only the synthetic AUD/USD `EmaCross`
  example.
- Result artifact format for golden trace comparison is not implemented yet.

These blockers should be closed by later RPROD/RCORE/RTRACE tasks, not bypassed
by Python fallback behavior.
