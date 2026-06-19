# v0.10.0 Dashboard Read-Only Order Proof Display

Date: 2026-06-19
Executor: Codex
Task: V100-009

## Plain Chinese Summary

这份文档记录 v0.10.0 Dashboard 现在可以只读展示 Binance testnet order proof
相关状态。它只显示风险预检、order-test、submit ack、cancel ack、终态 lifecycle
和 reconciliation 的证据状态，不提供任何下单、撤单、重试或生产交易按钮。

## Dashboard Surface

The local Dashboard workflow table includes a read-only `订单证明` column with:

- risk preflight status;
- order-test status;
- submit ack status;
- cancel ack status;
- terminal lifecycle status;
- reconciliation status;
- manual submit/cancel proof observed flag;
- testnet orders submitted/canceled counters;
- production orders submitted/canceled counters;
- Dashboard order controls flag.

## Artifact Inputs

The Dashboard reads these fields from manifest child artifacts when available:

```text
ntpro.v100_order_preflight_report.v1
ntpro.v100_order_test_preflight_report.v1
ntpro.v100_execution_artifact_contract.v1
ntpro.v100_submit_ack_artifact.v1
ntpro.v100_cancel_ack_artifact.v1
ntpro.v100_order_lifecycle_artifact.v1
ntpro.v100_reconciliation_artifact.v1
ntpro.v100_reconciliation_fixture_report.v1
```

Path fallback also recognizes:

```text
testnet_order_proof/risk_preflight.json
testnet_order_proof/order_test.json
testnet_order_proof/submit_ack.json
testnet_order_proof/cancel_ack.json
testnet_order_proof/lifecycle.json
testnet_order_proof/reconciliation.json
```

## Boundary

This display is read-only. It does not:

- submit orders;
- cancel orders;
- retry orders;
- connect to Binance;
- read Binance account/order state;
- enable Dashboard order controls;
- mark V100-006 manual submit/cancel proof as complete.

## Behavior Impact

Dashboard snapshots now expose additional read-only order proof fields. Existing
workflow artifact behavior remains unchanged when those fields are absent; the
new fields report `unknown`.

## Rollback

Revert the V100-009 PR to remove the Dashboard fields, parser additions, UI
column, tests, and this document.
