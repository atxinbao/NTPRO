# V05 Workflow Artifact User Guide

Date: 2026-06-13
Executor: Codex

## Purpose

This guide shows how to run the v0.5 local Binance sandbox workflow and inspect
the generated artifacts.

Plain Chinese summary: v0.5 把 v0.4 已经验证过的本地 Binance fixture replay、
EMA/RSI 策略 smoke、mock order lifecycle 和 risk rejection 证据串成一个 CLI
workflow。它会生成本地 JSON/JSONL 文件，方便用户和 Dashboard 读取。

This is sandbox-only local evidence. It does not connect to Binance, does not
use Binance testnet credentials, does not use real funds, does not submit real
orders, and does not prove production trading readiness.

## Run The Workflow

Build and run with Cargo:

```bash
cargo run -p nautilus-cli --bin nautilus -- workflow run \
  --workflow binance-sandbox \
  --run-id v05-smoke \
  --output /tmp/ntpro-v05-workflow-smoke
```

Expected output includes:

```text
workflow.run status=ok
workflow=binance-sandbox
external_venue_connection=false
real_funds=false
production_trading=false
real_orders_submitted=false
runtime_status=completed
```

## Artifact Directory

The output directory contains:

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

`manifest.json` is the completion marker. It references every artifact and
includes the dashboard-readable summary.

`boundary.json` records the safety boundary:

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

## Dashboard Readout

The local Dashboard reads workflow manifests from registry-adjacent
`workflows/*/manifest.json` directories and from `runs/workflows`.

The Dashboard surface is read-only:

- it displays run id, workflow, runtime status, manifest path, artifact count,
  evidence ids, and boundary booleans;
- it records invalid manifests as gaps;
- it does not start a workflow;
- it does not connect to Binance;
- it does not submit orders.

## Verify The Workflow

Use the explicit v0.5 smoke gate:

```bash
scripts/ai/verify_v05_workflow_artifacts.sh
```

The release gate also exposes this stage:

```bash
scripts/ai/verify_release.sh v05-workflow-artifacts-smoke
```

## What v0.5 Does Not Include

v0.5 does not include:

- Binance testnet runtime;
- real Binance account connectivity;
- real funds;
- production trading;
- real order submission;
- production Binance Spot or USDT-M parity;
- a remote or multi-user Dashboard.

Those items require later scoped tasks and separate release evidence.
