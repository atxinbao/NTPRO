# V06 Binance Testnet Dry-Run User Guide

Date: 2026-06-13
Executor: Codex

## Purpose

This guide explains the v0.6 Binance testnet dry-run foundation.

Plain Chinese summary: v0.6 新增的是 Binance testnet 的“离线 dry-run 基础”。它会
读取一个 testnet 配置文件，验证 credential policy、testnet URL、order lifecycle 和
reconciliation artifact，但不会连接 Binance，不会读取或保存真实 API key，不会提交订单。

This is not production trading and not real Binance testnet order submission.

## Config

Use the checked-in config:

```text
examples/rust/binance/testnet_dry_run.toml
```

The config stores environment variable names only:

```toml
[credentials]
api_key_env = "BINANCE_TESTNET_API_KEY"
api_secret_env = "BINANCE_TESTNET_API_SECRET"
values_in_file = false
required_for_network = true
```

Credential values must not be written into config files, artifacts, docs, or PR
bodies.

## Run The Dry-Run Workflow

```bash
cargo run -p nautilus-cli --bin nautilus -- workflow run \
  --workflow binance-testnet \
  --mode dry-run \
  --config examples/rust/binance/testnet_dry_run.toml \
  --run-id v06-smoke \
  --output /tmp/ntpro-v06-binance-testnet-dry-run
```

Expected output includes:

```text
workflow.run status=ok
workflow=binance-testnet
external_venue_connection=false
real_funds=false
production_trading=false
real_orders_submitted=false
testnet_connection=false
runtime_status=dry_run_completed
```

## Artifacts

The output directory contains:

```text
boundary.json
events.jsonl
manifest.json
orders/reconciliation.json
orders/testnet_dry_run_lifecycle.json
summary.json
testnet/config.json
testnet/connectivity_probe.json
testnet/credential_policy.json
```

Important fields:

```text
network_attempted=false
testnet_connection=false
values_recorded=false
real_orders_submitted=false
order_submission=disabled
reconciliation=artifact-only
```

## Dashboard

The local Dashboard reads `binance-testnet` workflow manifests through the
existing workflow artifact surface. It displays:

- workflow and run id;
- runtime status;
- manifest path and artifact count;
- Testnet / network / real funds / production trading / real order flags;
- credential policy and connectivity mode.

The Dashboard is read-only for v0.6. It does not start workflows, open network
connections, or submit orders.

## Verify

Use the v0.6 smoke:

```bash
scripts/ai/verify_v06_binance_testnet_dry_run.sh
```

The release gate stage is:

```bash
scripts/ai/verify_release.sh v06-binance-testnet-dry-run-smoke
```

## Not Included

v0.6 does not include:

- live Binance testnet network connection;
- real Binance testnet order submission;
- real account reconciliation;
- production Binance connectivity;
- real funds;
- production trading readiness.

Those require later scoped tasks and separate evidence.
