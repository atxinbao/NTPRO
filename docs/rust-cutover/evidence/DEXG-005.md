# DEXG-005 Integration Documentation Authority Evidence

Date: 2026-07-15
Executor: Codex
GitHub issue: #1084
Milestone: post-backend-docs-examples-governance
Status: LOCAL VALIDATION PASSED

## Summary

This evidence covers Rust-only authority normalization for integration pages.
It preserves venue facts and changes no adapter runtime behavior.

Plain Chinese summary: 本任务保留 integration 页中的 venue 协议、symbology、订单
能力和 rate-limit 信息，但明确所有 Python snippet 都是 retired upstream lineage，
不是当前 NTPRO 可运行入口。

## Affected Pages

- AX Exchange, Betfair, Binance, BitMEX, Bybit, Coinbase, Databento;
- Deribit, dYdX, Hyperliquid, Interactive Brokers, Kraken, OKX;
- Polymarket and Tardis.

## Validation

```text
integration authority audit = PASS
  affected pages = 15
  retained Python fences classified as lineage = 203
  mapped Rust adapter crates = 15
active optional-Python-binding claim search = PASS (no matches)
integration index backend-freeze/status boundary = PASS
scripts/ai/check_backend_freeze_baseline.sh = PASS
backend freeze negative selftest = PASS (20 cases)
scripts/ai/verify_fast.sh = PASS
frozen v0.32.0 release file diff = PASS (no changes)
Finder cache check = PASS (no .DS_Store)
git diff --check = PASS
```

## Behavior Impact

None. Documentation authority and status semantics only.
