# MVP 身份与追溯合同迁移说明

Date: 2026-08-02
Executor: Codex
Task: MVP-003

## 变更

`nautilus mvp serve` 现在要求节点配置显式包含 `[mvp]` 身份段，并在节点启动成功后
写出 `<workspace>/mvp/identity_contract.json`。该工件使用
`ntpro.mvp_identity_contract.v1` schema。

```toml
[mvp]
strategy_version = "v1"
backtest_run_id = "ema-cross-btcusdt-baseline-v1"
backtest_result_ref = "artifact://backtests/ema-cross-btcusdt-baseline-v1/summary.json"
account_id = "SANDBOX-001"
environment = "sandbox"
```

现有 `[strategy].strategy_id` 继续表示策略定义；`[node].node_id` 在 MVP 产品合同中
表示 `strategy_instance_id`；`mvp serve --node-id` 继续表示 Supervisor 管理的运行
节点。后两者必须不同，避免把进程边界误写成策略实例。

## 兼容性

- 直接运行 `ntpro-node` 的旧配置不受影响；
- 旧配置通过 `mvp serve` 启动时会因缺少 `[mvp]` 段而 fail closed；
- market 与 execution Venue 必须一致；
- MVP 环境只接受 `sandbox`；
- 新工件只读，不开放交易或节点控制能力。

## 回滚

回退 MVP-003 提交可恢复旧的 `mvp serve` 配置要求。回滚前应停止 MVP 进程；已有
`identity_contract.json` 是只读工件，可以随工作区一起删除。
