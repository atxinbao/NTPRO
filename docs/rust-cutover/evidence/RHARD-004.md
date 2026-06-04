# RHARD-004 Sandbox Demo Evidence

Date: 2026-06-04
Executor: Codex
Task ID: RHARD-004
Risk: medium

## Scope

RHARD-004 provides one minimal Rust CLI sandbox path with simulated data,
simulated execution, owner-visible lifecycle status, and local output
artifacts.

The implemented path:

- validates a small TOML sandbox config;
- uses synthetic quote event metadata;
- simulates sandbox order submission;
- writes `summary.txt` and `events.log`;
- reports node start and node stop;
- reports risk, portfolio, and cache state;
- does not connect to an external venue;
- does not submit real orders.

Full `LiveNode` runtime wiring and production adapter behavior remain
explicitly deferred.

## Input Data

The sandbox demo uses synthetic quote event metadata declared in:

```text
examples/rust/sandbox/sandbox_smoke.toml
```

Input section:

```toml
[[data]]
source = "synthetic-quotes"
instrument_id = "AUD/USD.SIM"
events = 3
```

## Config

The config file is:

```text
examples/rust/sandbox/sandbox_smoke.toml
```

Important safety fields:

```toml
[[venues]]
name = "SIM"
adapter = "sandbox"

[execution]
order_submission = "simulated"
reconciliation = "disabled"

[shutdown]
mode = "once"
```

The CLI rejects non-`sandbox` adapters for this demo path.

## Commands

Validation command:

```bash
source scripts/ai/toolchain_env.sh
cargo run -q -p nautilus-cli -- sandbox validate --config examples/rust/sandbox/sandbox_smoke.toml
```

Run command:

```bash
source scripts/ai/toolchain_env.sh
cargo run -q -p nautilus-cli -- sandbox run --config examples/rust/sandbox/sandbox_smoke.toml --run-id sandbox-smoke --output /tmp/ntpro-rhard-004-output
```

Output paths:

```text
/tmp/ntpro-rhard-004-output/summary.txt
/tmp/ntpro-rhard-004-output/events.log
```

## Results

Validation result:

```text
sandbox.validate status=ok mode=sandbox run_id=sandbox-smoke config=examples/rust/sandbox/sandbox_smoke.toml environment=sandbox trader_id=TRADER-001 venue_count=1 data_source=synthetic-quotes instrument_id=AUD/USD.SIM events=3 execution=simulated risk_state=simulated portfolio_state=simulated cache_state=in-memory external_venue_connection=false real_orders_submitted=false
```

Run result:

```text
sandbox.run status=ok mode=sandbox run_id=sandbox-smoke config=examples/rust/sandbox/sandbox_smoke.toml output=/tmp/ntpro-rhard-004-output summary=/tmp/ntpro-rhard-004-output/summary.txt events=/tmp/ntpro-rhard-004-output/events.log node_started=true node_stopped=true data_source=synthetic-quotes instrument_id=AUD/USD.SIM event_count=3 execution_state=simulated risk_state=simulated portfolio_state=simulated cache_state=in-memory external_venue_connection=false real_orders_submitted=false runtime_status=simulated_demo
```

Summary file:

```text
command=sandbox.run
status=ok
mode=sandbox
run_id=sandbox-smoke
config=examples/rust/sandbox/sandbox_smoke.toml
environment=sandbox
trader_id=TRADER-001
instance_id=sandbox-smoke-001
venue=SIM
adapter=sandbox
data_source=synthetic-quotes
instrument_id=AUD/USD.SIM
events=3
execution_state=simulated
risk_state=simulated
portfolio_state=simulated
cache_state=in-memory
node_started=true
node_stopped=true
shutdown_reason=once
external_venue_connection=false
real_orders_submitted=false
runtime_status=simulated_demo
```

Events file:

```text
event=validate_config status=ok
event=build_simulated_node status=ok trader_id=TRADER-001 instance_id=sandbox-smoke-001
event=node_start status=started environment=sandbox
event=market_data status=loaded source=synthetic-quotes instrument_id=AUD/USD.SIM events=3
event=risk_check status=passed mode=simulated max_order_qty=1000
event=execution status=simulated order_submission=simulated venue=SIM
event=portfolio_update status=simulated starting_balance=1000000 USD
event=cache_update status=simulated mode=in-memory warmup_instruments=AUD/USD.SIM
event=node_stop status=stopped shutdown_reason=once disconnect_timeout_secs=1
```

## Additional Validation

```bash
source scripts/ai/toolchain_env.sh
cargo run -q -p nautilus-cli -- sandbox run --help
cargo test -p nautilus-cli
git diff --check
scripts/ai/verify_fast.sh
scripts/ai/validate_agentflow_roles.py
```

Current results:

- `sandbox run --help`: passed and lists `--config`, `--run-id`, and `--output`.
- `cargo test -p nautilus-cli`: passed, 26 tests.
- Non-sandbox adapter unit check: passed, rejected `production-adapter`.
- `git diff --check`: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.
- `scripts/ai/validate_agentflow_roles.py`: passed.

## Behavior Impact

No real trading behavior changed. The new path is a local simulated sandbox
demo. It gives users a runnable CLI path for config validation, lifecycle
visibility, and artifact inspection while keeping real live-node runtime wiring
and adapter behavior deferred.

## Public API Impact

`nautilus sandbox validate` and `nautilus sandbox run` now execute the
RHARD-004 simulated demo config instead of returning a generic implementation
blocker.

The supported config is intentionally narrow:

- `run.mode = "sandbox"`;
- `run.environment = "sandbox"`;
- venue adapter must be `sandbox`;
- data source must be `synthetic-quotes`;
- execution order submission must be `simulated`;
- shutdown mode must be `once`.

## Missing Runtime Work

Still deferred:

- building a real `LiveNode` from the CLI config;
- wiring `nautilus_sandbox` execution components into CLI runtime;
- loading real fixture/catalog data;
- adapter support classification for production live mode;
- golden trace evidence for full live/sandbox runtime behavior.

## Rollback Plan

Revert this PR to remove the sandbox config parser, simulated demo output
writer, example config, docs updates, and RHARD-004 evidence.
