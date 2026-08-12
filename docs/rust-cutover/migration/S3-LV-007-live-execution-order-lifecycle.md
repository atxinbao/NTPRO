# S3-LV-007 Live 执行与订单生命周期迁移说明

Date: 2026-08-12
Executor: Codex

## 行为变化

S3-LV-006 的 `start_market_data` 行为不变。新增执行准入和执行启动动作，只有三方确认、风险
边界、幂等绑定和外部审计锚点全部通过时，才会构建带 Binance Spot 执行客户端的 LiveNode。

## 运维影响

- 继续使用 `NTPRO_BINANCE_LIVE_API_KEY` 和 `NTPRO_BINANCE_LIVE_API_SECRET`；
- owner、risk、operator 分别通过独立的机构用户、风控和运维 cookie 调用对应审批端点；不再
  接受同一浏览器会话代替三方审批，并需显式启用 `NTPRO_S3_LIVE_RUN_EXECUTION_SINGLE_SHOT=1`；
- `configs/nodes/btc-ema-shadow.toml#[risk]` 固定最大 Live 单笔名义金额和互异的 owner/risk/operator
  权威引用；客户端上限只能小于等于该值；
- 执行凭证必须是 Binance 当前执行客户端要求的 Ed25519 key；
- 节点启动前必须可访问 S3 外部审计锚点；配置、状态、签名回执或 Runtime artifact root 漂移均拒绝
  注册执行客户端；
- 默认部署不提供任何准入，因此升级后不会自动发送订单；
- 未知、超时或部分成交状态会阻断新订单，需要人工停止和核对；
- 回滚到 S3-LV-006 后执行准入和订单操作不可用，生产行情只读路径仍可使用。

## API 兼容性

Live Run action 枚举增加 `start_execution`，响应增加 `execution_order` 和三方审批状态；旧的
`execution-admission` POST 被三个 `execution-approvals/{owner|risk|operator}` 端点替代。严格
客户端必须重新从 OpenAPI 生成；旧客户端不会自动触发新增动作。取消和改单仍未开放，后续
必须通过独立任务扩展。
