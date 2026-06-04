# RHARD-005 Live Init Smoke Evidence

Date: 2026-06-04
Executor: Codex
Task ID: RHARD-005
Risk: medium

## Scope

RHARD-005 verifies a Rust live-node initialization and shutdown path without
real orders, real credentials, or production trading endpoints.

The implemented path is a Cargo example:

```text
crates/live/examples/live_init_smoke.rs
```

It:

- builds a Rust `LiveNode` in `Sandbox` mode;
- registers the `nautilus_sandbox` simulated execution client;
- starts the node;
- confirms the execution engine connects;
- confirms the sandbox account is present in cache;
- stops the node;
- confirms the execution engine disconnects;
- reports `real_orders_submitted=false`;
- reports `external_venue_connection=false`.

The `nautilus live` CLI remains deferred. RHARD-005 does not wire the live CLI
to the runtime.

## Config

The owner-visible equivalent config is recorded in:

```text
examples/rust/live/live_init_smoke.toml
```

Important safety fields:

```toml
[adapter]
kind = "sandbox-simulated-execution"

[execution]
order_submission = "disabled"
reconciliation = false
external_venue_connection = false
```

## Command

```bash
source scripts/ai/toolchain_env.sh
cargo run -q -p nautilus-live --no-default-features --features node --example live-init-smoke
```

## Result

The command passed. Key owner-visible lines:

```text
phase=build_node status=ok node_name=LiveInitSmoke
phase=register_adapter status=ok adapter=sandbox client_id=SANDBOX
phase=pre_start state=Idle exec_connected=false real_orders_submitted=false external_venue_connection=false
phase=start status=ok state=Running running=true exec_connected=true account_cached=true
phase=stop status=ok state=Stopped running=false exec_disconnected=true real_orders_submitted=false external_venue_connection=false rust_only_runtime=true
```

The command also emitted runtime logs showing:

- `Sandbox execution client started: venue=SANDBOX`;
- `All engine clients connected`;
- `Startup reconciliation disabled`;
- `Trader started`;
- `Sandbox execution client disconnected: venue=SANDBOX`;
- `All engine clients disconnected`;
- `Sandbox execution client stopped: venue=SANDBOX`.

## Additional Validation

```bash
source scripts/ai/toolchain_env.sh
cargo check -p nautilus-live --no-default-features --features node --example live-init-smoke
git diff --check
scripts/ai/validate_agentflow_roles.py
scripts/ai/verify_fast.sh
```

Current results:

- `cargo check -p nautilus-live --no-default-features --features node --example live-init-smoke`: passed.
- `cargo run -q -p nautilus-live --no-default-features --features node --example live-init-smoke`: passed.
- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.

## Verification Note

The first `cargo run` attempt left a stale Cargo build lock. The stale `cargo`
process was terminated after confirming there was no active `rustc` or example
process. The command was rerun successfully after the lock was released.

A supplemental targeted test command was also attempted:

```bash
cargo test -p nautilus-live --no-default-features --features node --test node test_rust_sandbox_execution_client_start_stop_smoke_rust_only -- --nocapture
```

It was not used as RHARD-005 evidence because the Cargo process stalled while
holding the build lock and had no active `rustc` child process. It was
terminated to avoid blocking the workspace. The required executable smoke for
RHARD-005 is the passing `live-init-smoke` example above.

## Behavior Impact

No production behavior changed. This is a Rust live-node smoke example using
the sandbox simulated execution client. It proves the local Rust initialization
and shutdown path but does not enable live CLI execution or production adapter
behavior.

## Public API Impact

No public API is changed. A new Cargo example is added:

```text
cargo run -p nautilus-live --no-default-features --features node --example live-init-smoke
```

## Missing Runtime Work

Still deferred:

- `nautilus live validate` config parsing;
- `nautilus live run` runtime wiring;
- production adapter support classification;
- live data client fixture strategy;
- real adapter connection lifecycle;
- golden trace evidence for live runtime parity.

## Rollback Plan

Revert this PR to remove the live init Cargo example, equivalent config,
documentation updates, RHARD-005 evidence, and agentflow state changes.
