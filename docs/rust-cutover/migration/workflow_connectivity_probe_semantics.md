# Workflow connectivity-probe semantics

Date: 2026-06-14
Executor: Codex

## Summary

In v0.6.1, Binance testnet `connectivity-probe` is an offline contract check.
It records user intent and artifact semantics only. It does not open a network
socket and does not prove real Binance testnet connectivity.

## Fields

When running:

```bash
nautilus workflow run \
  --workflow binance-testnet \
  --mode connectivity-probe \
  --allow-testnet-network \
  --config examples/rust/binance/testnet_dry_run.toml
```

the workflow records:

- `requested_mode=connectivity-probe`;
- `network_permission_requested=true`;
- `network_attempted=false`;
- `testnet_connection=false`;
- `runtime_status=offline_probe_validated`.

These values appear in CLI output, `summary.json`, `manifest.json.summary`, and
`testnet/connectivity_probe.json`.

## User Impact

Consumers must not treat `offline_probe_validated` as real exchange
connectivity. It means the local contract and requested mode were validated
without opening a network connection.

Real read-only Binance testnet connectivity belongs to the later v0.7.0 gate and
must require explicit online opt-in and credentials.
