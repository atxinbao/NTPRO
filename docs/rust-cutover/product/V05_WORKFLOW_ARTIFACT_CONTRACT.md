# V05 Workflow Artifact Contract

Date: 2026-06-13
Executor: Codex

## Purpose

`v0.5` introduces a local Binance sandbox workflow artifact path for the Rust CLI.

Plain language: `nautilus workflow run --workflow binance-sandbox` stitches the
already evidenced v0.4 local pieces into one local artifact directory. It uses
checked-in Binance bar fixtures, deterministic strategy smoke summaries, mock
order lifecycle evidence, and deterministic risk rejection evidence.

This is not a real Binance connection, not testnet, not production trading, and
not real order submission.

## CLI Contract

```bash
cargo run -p nautilus-cli --bin nautilus -- workflow run \
  --workflow binance-sandbox \
  --run-id v05-smoke \
  --output /tmp/ntpro-v05-workflow-smoke
```

Expected stdout contains:

```text
workflow.run status=ok
workflow=binance-sandbox
external_venue_connection=false
real_funds=false
production_trading=false
real_orders_submitted=false
runtime_status=completed
```

## Artifact Contract

The workflow writes these files under the output directory:

```text
boundary.json
events.jsonl
manifest.json
market/replay.json
orders/mock_lifecycle.json
risk/rejection.json
strategies/ema.json
strategies/rsi.json
summary.json
```

`manifest.json` is written last. It uses schema version
`ntpro.workflow_manifest.v1` and references all generated artifacts.

`summary.json` uses schema version `ntpro.workflow_summary.v1` and carries the
dashboard-ready product summary.

`boundary.json` uses schema version `ntpro.workflow_boundary.v1` and records:

```text
sandbox_only=true
fixture_replay=true
mock_execution=true
external_venue_connection=false
real_funds=false
production_trading=false
real_orders_submitted=false
testnet_connection=false
```

## Boundary

Allowed:

- local checked-in fixture replay;
- local deterministic strategy smoke summaries;
- local mock order lifecycle summary;
- local deterministic risk rejection summary;
- dashboard-readable JSON/JSONL artifacts.

Not allowed:

- real Binance venue connection;
- Binance testnet connection;
- real funds;
- production trading;
- real order submission;
- claims that this workflow proves live production readiness.

## Rollback

Revert the PR that added `crates/cli/src/workflow.rs`, the CLI routing, and this
contract. No persisted trading state or external venue state is touched.
