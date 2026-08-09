# S1-BT-005 Backtest 比较与复现合同迁移说明

Date: 2026-08-09
Executor: Codex

## 新增合同

`GET /api/product/v1/run-comparisons?run_ids=<id1>,<id2>` 接受 2 至 4 个唯一 Backtest Run。
响应按请求顺序返回参数、指标、风险和来源，并明确 `directly_comparable`。只有数据 SHA、标的
和币种都一致时才可直接比较；策略版本不同会单独标记，但不会隐藏结果。

`POST /api/product/v1/runs/{run_id}/reproduction` 必须提交：

```json
{"source_run_id":"<run_id>","deterministic_replay":true}
```

服务端从动态 Run 的不可变 `request.toml` 重建输入，重新执行 `BacktestEngine`，创建新的 Run
并写入 `reproduction.json`。`GET /api/product/v1/runs/{new_run_id}/reproduction` 返回经重新
计算验证的输入与输出等价证明。

每个新动态 Run 同时写入 `strategy-version.json`，并在 Run manifest 中绑定其 SHA-256。
比较和复现按每个 Run 自己的不可变版本快照验证，因此当前默认版本升级后，旧版本 Run 仍可
读取、比较和复现；快照缺失、哈希漂移或内容哈希不匹配均 fail closed。

## 兼容行为

Run 的 `result.reproduction_ref` 是新增的必需可空字段。历史 Run 返回 `null`；旧的 detail、
metrics、report 和 analysis 语义不变。客户端不得自行拼接证明路径，只能在该字段非空时读取。
source-controlled 静态历史 Run 继续按当前可信 StrategyVersion 读取。任何动态 Run 都必须同时
具备 `strategy-version.json` 和 manifest SHA-256 绑定；S1-BT-005 之前没有版本快照的本地动态
产物不再被 Product API 信任，必须使用当前版本重新创建 Run。删除快照或 manifest 绑定不能
降级回退到当前版本。

## 边界

复现是用户主动创建新 Backtest Run，不是 retry 或 remediation。该合同不开放 Demo、Live、
外部 Venue、订单提交、订单修改、自动恢复或交易控件。
