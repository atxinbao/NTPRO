# Live and Sandbox CLI Run Contract

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-005

## Purpose

This contract refines the `nautilus sandbox` and `nautilus live` surfaces from
`docs/rust-cutover/product/RUST_CLI_CONTRACT.md`. It defines the Rust-first
command shape, config boundary, lifecycle, shutdown behavior, output contract,
and known blockers for later implementation tasks.

This is a product contract only. RPROD-005 does not implement the commands.

## Current Baseline

After RPROD-004, `nautilus-cli` exposes `database` and `backtest` by default.
The `backtest` command has help and parser coverage, but its execution path is
intentionally blocked until later runtime tasks.

The live and sandbox product commands are not implemented yet:

```text
nautilus sandbox --help
nautilus live --help
```

Current expected result: both commands exit non-zero with an unknown subcommand
error. This is an owner-visible blocker until RPROD-006 or a later scoped task
adds the CLI implementation.

## Command Surface

The Rust-first sandbox command must expose:

```text
nautilus sandbox validate --config <path>
nautilus sandbox run --config <path> [--run-id <id>] [--output <dir>]
```

The Rust-first live command must expose:

```text
nautilus live validate --config <path>
nautilus live run --config <path> [--run-id <id>] [--output <dir>]
```

`validate` and `run` must parse the same config shape for each mode. `validate`
must not start a live node, connect to an external venue, or send adapter
commands. `run` must perform validation first, then continue only if the config
maps cleanly into the Rust live-node boundary.

## Sandbox Mode

Sandbox mode is the first preferred run target for Rust-first lifecycle smoke.
It must use Rust live-node construction with sandbox execution components and
must not send orders to production venues.

Allowed Rust integration points:

- `nautilus_live::node::LiveNode`;
- `nautilus_live::node::LiveNodeBuilder`;
- `nautilus_live::config::LiveNodeConfig`;
- `nautilus_sandbox` adapter components;
- fixture or local deterministic data components approved by adapter tasks.

`sandbox validate` must check:

- config file exists and parses;
- `run.id` or `--run-id` is non-empty;
- trader and instance identifiers are syntactically valid;
- sandbox venue and account sections are present;
- at least one data input or fixture source is configured when required by the
  selected sandbox flow;
- shutdown settings are syntactically valid;
- output settings are syntactically valid.

`sandbox run` must emit startup and shutdown status. It may remain blocked until
a scoped runtime smoke task proves a deterministic sandbox node lifecycle.

## Live Mode

Live mode is the Rust-first entrypoint for scoped adapter live-node workflows.
It must not become the default evidence path before adapter support is
classified and a fixture, dry-run, sandbox, or explicitly scoped live adapter
strategy exists.

Allowed Rust integration points:

- `nautilus_live::node::LiveNode`;
- `nautilus_live::node::LiveNodeBuilder`;
- `nautilus_live::config::LiveNodeConfig`;
- adapter-specific Rust factory/config types that have RADP evidence.

`live validate` must check:

- config file exists and parses;
- `run.id` or `--run-id` is non-empty;
- trader and instance identifiers are syntactically valid;
- live venue and adapter sections are present;
- each selected adapter is classified as supported, deferred, or blocked;
- credentials and connection fields are declared through documented config or
  environment variables, without printing secret values;
- reconciliation and startup settings map to Rust config structs;
- shutdown settings are syntactically valid;
- output settings are syntactically valid.

`live run` must perform the same validation before building a node. Production
adapter behavior must remain blocked unless an adapter task explicitly scopes
that behavior and provides fixture, dry-run, sandbox, or live evidence.

## Config Format

The first implementation should use TOML, matching the backtest CLI contract.
JSON can be added later only as an explicit extension.

Minimum sandbox TOML shape:

```toml
[run]
id = "sandbox-smoke"
mode = "sandbox"
environment = "sandbox"

[system]
trader_id = "TRADER-001"
instance_id = "sandbox-smoke-001"
log_level = "info"

[[venues]]
name = "SIM"
adapter = "sandbox"
account_type = "MARGIN"
oms_type = "HEDGING"
starting_balances = ["1000000 USD"]

[[data]]
source = "fixture"
catalog_path = "catalog/sandbox-smoke"
instrument_id = "AUD/USD.SIM"

[execution]
reconciliation = "disabled"
startup_timeout_secs = 120

[shutdown]
mode = "duration"
max_runtime_secs = 30
disconnect_timeout_secs = 10

[output]
dir = "runs/sandbox-smoke"
write_summary = true
```

Minimum live TOML shape:

```toml
[run]
id = "live-dry-run"
mode = "live"
environment = "sandbox"

[system]
trader_id = "TRADER-001"
instance_id = "live-dry-run-001"
log_level = "info"

[[venues]]
name = "BINANCE"
adapter = "binance"
account_type = "MARGIN"
oms_type = "HEDGING"
connection_profile = "sandbox"

[[data]]
source = "adapter"
venue = "BINANCE"
instrument_id = "BTCUSDT.BINANCE"

[execution]
reconciliation = "startup"
startup_timeout_secs = 120
allow_order_submission = false

[shutdown]
mode = "signal"
disconnect_timeout_secs = 10

[output]
dir = "runs/live-dry-run"
write_summary = true
```

### Field Mapping

`run` identifies the workflow and must map to an owner-visible run ID.

`system` maps to live-node and kernel identity/config fields.

`venues` maps to sandbox or adapter-specific venue factories. Unsupported
adapters must be rejected explicitly.

`data` maps to fixture, catalog, or adapter data inputs. Adapter data-provider
behavior remains under RADP tasks.

`execution` maps to reconciliation, startup timeout, and order-submission
policy. Live order submission must be opt-in by config and adapter scope, not an
implicit default.

`shutdown` maps to lifecycle stop conditions:

- `signal`: run until SIGINT, SIGTERM, or an internal shutdown command;
- `duration`: run until `max_runtime_secs`;
- `once`: start, perform one scoped smoke cycle, then shut down.

`output` controls owner-visible artifacts and must not affect trading
semantics.

## Lifecycle Contract

Both `sandbox run` and `live run` must report these phases when implemented:

```text
validate_config
build_node
connect_clients
start_trader
run_until_stop_condition
stop_trader
disconnect_clients
write_summary
```

Failure in any phase must return a non-zero exit and name the failing phase.
Unsupported config sections must be rejected explicitly instead of ignored.

## Output Contract

`validate` must print a concise success or failure summary:

```text
sandbox.validate status=ok config=<path> run_id=<id>
live.validate status=ok config=<path> run_id=<id>
```

`run` must print or write:

- command name;
- run ID;
- config path;
- output directory;
- lifecycle phase;
- started timestamp;
- completed or failed timestamp;
- final status;
- venue count;
- adapter support decision summary;
- shutdown reason.

Human-readable text is enough for the initial implementation. Machine-readable
JSON output can be added later as an explicit `--format json` option.

## Failure Behavior

The commands must use stable non-zero exits so automation can distinguish user
errors from runtime failures.

Recommended exit codes:

- `2`: CLI usage or argument parse error;
- `3`: config parse or validation error;
- `4`: adapter support blocked or unclassified;
- `5`: node build error;
- `6`: startup or connection error;
- `7`: runtime lifecycle error;
- `8`: shutdown or disconnect error;
- `9`: output artifact write error.

## Implementation Gates

The command surface is not considered usable until all of the following pass:

```bash
cargo run -q -p nautilus-cli -- sandbox --help
cargo run -q -p nautilus-cli -- sandbox validate --help
cargo run -q -p nautilus-cli -- sandbox run --help
cargo run -q -p nautilus-cli -- live --help
cargo run -q -p nautilus-cli -- live validate --help
cargo run -q -p nautilus-cli -- live run --help
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
```

The first lifecycle smoke must also prove:

- no Python import is required;
- no PyO3 or Cython build artifact is required;
- a Rust sandbox or live node can build, start, and stop;
- the run emits an owner-visible run ID, lifecycle status, and shutdown reason.

## Known Blockers

- `nautilus sandbox` is not implemented in the current CLI.
- `nautilus live` is not implemented in the current CLI.
- A Rust CLI config parser and TOML model have not been added for live-node
  workflows.
- Sandbox lifecycle smoke is not wired into the CLI.
- Adapter support for live mode is not classified by the CLI.
- Production live adapter behavior requires adapter evidence and explicit task
  scope before it can be used as release evidence.

These blockers should be closed by later RPROD, RCORE, RADP, and RTRACE tasks,
not bypassed by Python fallback behavior.
