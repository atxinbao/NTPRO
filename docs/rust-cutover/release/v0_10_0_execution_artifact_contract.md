# v0.10.0 Execution Artifact Contract

Date: 2026-06-19
Executor: Codex
Task: V100-007

## Plain Chinese Summary

这份文档定义 v0.10.0 后续“真实 Binance testnet 小额下单再撤单”需要留下哪些证据文件。
本任务只定义证据格式和离线 JSON 合约，不联网、不发单、不撤单，也不表示 V100-006
已经完成。

## Contract

The CLI command is:

```bash
nautilus live testnet-execution-artifact-contract \
  --config configs/nodes/btc-ema-shadow.toml \
  --timestamp-ms 1718400000000 \
  --output runs/v100/execution-artifact-contract.json \
  --allow-testnet-order \
  --confirm-owner-approved-testnet-order \
  --confirm-tiny-notional \
  --confirm-cancel-after-submit
```

The output schema is:

```text
ntpro.v100_execution_artifact_contract.v1
```

The contract defines these artifact slots:

- `request.json`
- `order_test.json`
- `submit_ack.json`
- `cancel_ack.json`
- `lifecycle.json`
- `reconciliation.json`

Offline evidence must preserve these values:

```text
manual_submit_cancel_proof_observed=false
matching_engine_submission=false
order_submission_remains_disabled=true
network_attempted=false
real_orders_submitted=false
production_endpoint_allowed=false
dashboard_order_controls=false
secrets_redacted=true
```

Counters are separated by venue boundary:

```text
testnet_orders_submitted=0
testnet_orders_canceled=0
production_orders_submitted=0
production_orders_canceled=0
```

## Manual Proof Boundary

V100-006 remains separate. A future owner-approved online proof must use Binance
testnet only, tiny notional only, immediate cancel-after-submit, and redacted
artifacts. This contract does not authorize production Binance or real funds.

## Redaction Boundary

The contract must not persist:

- API key values;
- API secrets;
- signatures;
- signed query strings;
- signed URLs;
- request bodies with sensitive values.

## Behavior Impact

This is an additive local artifact contract. It does not change strategy
runtime behavior, matching-engine behavior, adapter behavior, or Dashboard
controls.
