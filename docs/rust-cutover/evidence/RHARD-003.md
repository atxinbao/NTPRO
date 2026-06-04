# RHARD-003 CLI Help Contract Evidence

Date: 2026-06-04
Executor: Codex
Task ID: RHARD-003
Risk: medium

## Scope

RHARD-003 stabilizes and documents the Rust CLI help-level product contract for
the v0.2.0 hardening phase.

This task does not implement full backtest, sandbox, live, data, or config
runtime behavior. It records which command surfaces are help-stable and which
runtime paths remain deferred.

## Context Reviewed

- `docs/rust-cutover/tasks/RHARD-003.md`
- `docs/rust-cutover/product/RUST_CLI_CONTRACT.md`
- `docs/rust-cutover/product/BACKTEST_CLI_CONTRACT.md`
- `docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md`
- `docs/rust-cutover/product/DATA_CATALOG_CLI_CONTRACT.md`
- `docs/rust-cutover/product/CONFIG_VALIDATION_CLI_CONTRACT.md`
- `docs/rust-cutover/verification/README.md`
- `crates/cli/src/opt.rs`
- `crates/cli/src/lib.rs`
- `crates/cli/src/bin/cli.rs`

## Changes

- Added `docs/rust-cutover/product/CLI_HELP_CONTRACT.md`.
- Updated `docs/rust-cutover/product/RUST_CLI_CONTRACT.md` so its current CLI
  baseline matches the live Rust CLI help surface.
- Added database help and parser coverage in `crates/cli/src/opt.rs`.
- Recorded supported, deferred, and out-of-scope CLI contracts.

## Toolchain Note

A direct `cargo run` attempt used Homebrew Rust `1.87.0` and failed because
NTPRO requires Rust `1.95.0`. The required help commands were rerun after
sourcing `scripts/ai/toolchain_env.sh`, which pinned Cargo/Rust `1.95.0`.

## Commands Run

```bash
source scripts/ai/toolchain_env.sh
cargo run -p nautilus-cli -- --help
cargo run -p nautilus-cli -- backtest --help
cargo run -p nautilus-cli -- sandbox --help
cargo run -p nautilus-cli -- live --help
cargo run -p nautilus-cli -- data --help
cargo run -p nautilus-cli -- config --help
cargo run -p nautilus-cli -- database --help
cargo run -p nautilus-cli -- backtest validate --help
cargo run -p nautilus-cli -- backtest run --help
cargo run -p nautilus-cli -- sandbox validate --help
cargo run -p nautilus-cli -- sandbox run --help
cargo run -p nautilus-cli -- live validate --help
cargo run -p nautilus-cli -- live run --help
cargo run -p nautilus-cli -- data inspect --help
cargo run -p nautilus-cli -- data validate --help
cargo run -p nautilus-cli -- data load --help
cargo run -p nautilus-cli -- config validate --help
cargo run -p nautilus-cli -- database init --help
cargo run -p nautilus-cli -- database drop --help
cargo test -p nautilus-cli
git diff --check
scripts/ai/verify_fast.sh
scripts/ai/validate_agentflow_roles.py
```

## Captured Help Surface

Top-level help passed and listed:

```text
Commands:
  backtest  Backtest operations
  sandbox   Sandbox live-node operations
  live      Live trading operations
  data      Data catalog operations
  config    Rust config validation
  database  Postgres database operations
```

Backtest help passed:

```text
Usage: nautilus backtest <COMMAND>

Commands:
  validate  Validates a Rust backtest config without running the engine
  run       Runs a Rust backtest from a validated config
```

Sandbox help passed:

```text
Usage: nautilus sandbox <COMMAND>

Commands:
  validate  Validates a Rust sandbox config without starting a node
  run       Runs a Rust sandbox live-node flow from a validated config
```

Live help passed:

```text
Usage: nautilus live <COMMAND>

Commands:
  validate  Validates a Rust live config without starting a node
  run       Runs a Rust live-node flow from a validated config
```

Data help passed:

```text
Usage: nautilus data <COMMAND>

Commands:
  inspect   Inspects catalog or source metadata without running a strategy
  validate  Validates catalog availability and requested data windows
  load      Loads scoped source data into a configured catalog target
```

Config help passed:

```text
Usage: nautilus config <COMMAND>

Commands:
  validate  Validates a Rust workflow config without running the workflow
```

Database help passed:

```text
Usage: nautilus database <COMMAND>

Commands:
  init  Initializes a new Postgres database with the latest schema
  drop  Drops roles, privileges and deletes all data from the database
```

## Supported Contracts

- `nautilus --help`
- `nautilus backtest --help`
- `nautilus sandbox --help`
- `nautilus live --help`
- `nautilus data --help`
- `nautilus config --help`
- `nautilus database --help`
- all documented second-level subcommand help surfaces in
  `docs/rust-cutover/product/CLI_HELP_CONTRACT.md`

## Deferred Contracts

The following commands are help-stable but runtime-deferred:

- `backtest validate`
- `backtest run`
- `sandbox validate`
- `sandbox run`
- `live validate`
- `live run`
- `data inspect`
- `data validate`
- `data load`
- `config validate`

They currently return explicit Rust blocker errors from `crates/cli/src/lib.rs`
instead of falling back to Python.

## Missing or Out of Scope

- Full backtest, sandbox, live, data, and config runtime behavior.
- Machine-readable CLI output.
- Release binaries or installers.
- Default-build `blockchain` help; it remains feature-gated behind `defi`.
- Python CLI entrypoints.

## Behavior Impact

No trading semantics changed. Runtime behavior remains blocked where it was
already blocked. The code change adds parser tests for the existing database
CLI surface only.

## Results

- Required CLI help commands: passed after sourcing
  `scripts/ai/toolchain_env.sh`.
- Supplemental second-level help commands: passed.
- `cargo test -p nautilus-cli`: passed, 20 tests.
- `git diff --check`: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.
- `scripts/ai/validate_agentflow_roles.py`: passed.

## Rollback Plan

Revert this PR to remove the RHARD-003 help contract document, evidence file,
database parser tests, and CLI baseline update.
