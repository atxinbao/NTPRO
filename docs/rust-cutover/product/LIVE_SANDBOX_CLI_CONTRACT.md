# Live and Sandbox CLI Run Contract

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-005

Updated: 2026-06-06
Executor: Codex
Task ID: DRG-005

## Purpose

This contract refines the `nautilus sandbox` and `nautilus live` surfaces from
`docs/rust-cutover/product/RUST_CLI_CONTRACT.md`. It defines the Rust-first
command shape, config boundary, lifecycle, shutdown behavior, output contract,
and known blockers for later implementation tasks.

This document started as a product contract. RHARD-004 later added a local
simulated sandbox demo through `nautilus sandbox validate` and
`nautilus sandbox run`. DRG-005 adds the first scoped `nautilus live run`
LiveNode start/stop smoke through the sandbox simulated execution client.

## Current Baseline

The current CLI exposes `sandbox` and `live` by default.

Current capability status is recorded in
`docs/rust-cutover/product/CLI_CAPABILITY_MATRIX.md`.

- `nautilus sandbox validate` validates the RHARD-004 local simulated demo
  config.
- `nautilus sandbox run` writes deterministic simulated demo artifacts.
- `nautilus live validate` validates the DRG-005 live-init smoke config.
- `nautilus live run` starts and stops a Rust `LiveNode` in sandbox mode with
  the simulated execution client registered. It does not connect to a real
  venue or submit real orders.

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

`sandbox run` must not imply a real node lifecycle until real `LiveNode` wiring
exists. The RHARD-004 CLI path writes simulated demo artifacts and reports
`runtime_status=simulated_demo`, `live_node_started=false`, and
`live_node_stopped=false`.

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
source = "synthetic-quotes"
instrument_id = "AUD/USD.SIM"
events = 3

[execution]
order_submission = "simulated"
reconciliation = "disabled"

[risk]
mode = "simulated"
max_order_qty = 1000

[portfolio]
mode = "simulated"
starting_balance = "1000000 USD"

[cache]
mode = "in-memory"
warmup_instruments = ["AUD/USD.SIM"]

[shutdown]
mode = "once"
max_runtime_secs = 1
disconnect_timeout_secs = 1

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

DRG-005 records the live init smoke config at
`examples/rust/live/live_init_smoke.toml`. That config uses the sandbox
simulated execution client, disables order submission, and is executed through
`nautilus live validate` and `nautilus live run`.

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

The RHARD-004 sandbox demo also writes `summary.txt` and `events.log` with a
simulated lifecycle, synthetic data flow, simulated execution, risk, portfolio,
and cache status. It explicitly reports `live_node_started=false`,
`live_node_stopped=false`, `external_venue_connection=false`, and
`real_orders_submitted=false`.

The DRG-005 live-init smoke writes `summary.txt` and `events.log` after
starting and stopping a Rust `LiveNode` in sandbox mode. It reports
`runtime_status=completed`, `external_venue_connection=false`, and
`real_orders_submitted=false`.

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
cargo run -q -p nautilus-cli -- live validate --config examples/rust/live/live_init_smoke.toml
cargo run -q -p nautilus-cli -- live run --config examples/rust/live/live_init_smoke.toml --output runs/live-init-smoke
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
```

The first lifecycle smoke must also prove:

- no Python import is required;
- no PyO3 or Cython build artifact is required;
- a Rust sandbox or live node can build, start, and stop;
- the run emits an owner-visible run ID, lifecycle status, and shutdown reason.

## Remaining Blockers

- Adapter support for live mode is not classified by the CLI.
- Production live adapter behavior requires adapter evidence and explicit task
  scope before it can be used as release evidence.
- The DRG-005 live path supports only sandbox simulated execution with order
  submission, reconciliation, and external venue connections disabled.

RHARD-004 closes the sandbox CLI blocker for the local simulated demo only.
DRG-005 closes the first live CLI start/stop smoke only. Production adapter
behavior remains deferred.

These blockers should be closed by later RPROD, RCORE, RADP, and RTRACE tasks,
not bypassed by Python fallback behavior.
