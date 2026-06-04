# RHARD-006 Backtest CLI Minimal Path Evidence

Date: 2026-06-04
Executor: Codex
Task ID: RHARD-006
Risk: medium

## Scope

RHARD-006 provides one minimal Rust CLI backtest path with input, config,
command, output path, and expected result.

The implemented path is metadata-only:

- it validates a small TOML config;
- it writes a summary file;
- it does not start `BacktestEngine`;
- it does not load market data into the runtime;
- it does not run a trading strategy;
- it does not change trading semantics.

Full backtest runtime wiring remains explicitly deferred.

## Input Data

The minimal path uses synthetic quote metadata declared in:

```text
examples/rust/backtest/minimal_dry_run.toml
```

Input section:

```toml
[data]
source = "synthetic-quotes"
instrument_id = "AUD/USD.SIM"
quotes = 3
```

This input is intentionally metadata-only for RHARD-006. The CLI does not
materialize quote ticks or start a runtime engine in this task.

## Config

The config file is:

```text
examples/rust/backtest/minimal_dry_run.toml
```

Required shape:

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

The only currently supported strategy selector is `no-op`.

## Commands

Validation command:

```bash
source scripts/ai/toolchain_env.sh
cargo run -q -p nautilus-cli -- backtest validate --config examples/rust/backtest/minimal_dry_run.toml
```

Run command:

```bash
source scripts/ai/toolchain_env.sh
cargo run -q -p nautilus-cli -- backtest run --config examples/rust/backtest/minimal_dry_run.toml --dry-run --output /tmp/ntpro-rhard-006-output
```

Output path:

```text
/tmp/ntpro-rhard-006-output/summary.txt
```

## Results

Validation result:

```text
backtest.validate status=ok mode=dry-run run_id=minimal-backtest-dry-run config=examples/rust/backtest/minimal_dry_run.toml input=synthetic-quotes instrument_id=AUD/USD.SIM quotes=3 strategy=no-op
```

Run result:

```text
backtest.run status=ok mode=dry-run run_id=minimal-backtest-dry-run config=examples/rust/backtest/minimal_dry_run.toml input=synthetic-quotes instrument_id=AUD/USD.SIM quotes=3 strategy=no-op output=/tmp/ntpro-rhard-006-output summary=/tmp/ntpro-rhard-006-output/summary.txt engine_started=false runtime_status=deferred
```

Summary file:

```text
command=backtest.run
status=ok
mode=dry-run
run_id=minimal-backtest-dry-run
config=examples/rust/backtest/minimal_dry_run.toml
input=synthetic-quotes
instrument_id=AUD/USD.SIM
quotes=3
strategy=no-op
engine_started=false
runtime_status=deferred
```

Runtime blocker check:

```bash
cargo run -q -p nautilus-cli -- backtest run --config examples/rust/backtest/minimal_dry_run.toml
```

Result: exited non-zero and reported that full backtest runtime wiring is not
implemented yet. This confirms RHARD-006 does not hide missing runtime work.

## Additional Validation

```bash
source scripts/ai/toolchain_env.sh
cargo run -q -p nautilus-cli -- backtest run --help
cargo test -p nautilus-cli
git diff --check
scripts/ai/verify_fast.sh
scripts/ai/validate_agentflow_roles.py
```

Current results:

- `backtest run --help`: passed and lists `--dry-run`.
- `cargo test -p nautilus-cli`: passed, 23 tests.
- Runtime blocker check without `--dry-run`: passed, exited non-zero with an
  explicit blocker.
- `git diff --check`: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.
- `scripts/ai/validate_agentflow_roles.py`: passed.

## Behavior Impact

No trading behavior changed. The new CLI path is explicitly metadata-only and
does not start the backtest engine. It adds a safe user-visible path for
checking config shape and output wiring while later tasks implement real runtime
execution.

## Public API Impact

`nautilus backtest run --help` now includes:

```text
--dry-run
```

This flag is additive and scoped to the RHARD-006 metadata-only minimal path.

## Missing Runtime Work

Still deferred:

- TOML mapping into `BacktestRunConfig`;
- strategy registry or built-in strategy execution;
- catalog/data loading from config;
- `BacktestEngine` or `BacktestNode` startup from CLI;
- golden trace result artifact emission for full runtime comparison.

## Rollback Plan

Revert this PR to remove the minimal dry-run config parser, example config,
contract updates, and RHARD-006 evidence.
