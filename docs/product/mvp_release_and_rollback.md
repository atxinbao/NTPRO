# NTPRO 单节点 MVP 发布与人工回滚说明

Date: 2026-08-05
Executor: Codex
Status: FROZEN ON MVP-013 MERGE

## 产品基线

本基线只交付本地单 Supervisor + 单 `ntpro-node` + 单策略实例 + 单沙盒账户/Venue 的
MVP。机构工作台负责只读业务状态，控制中心负责单节点状态、诊断和 operator 显式
start/stop。权威机器合同是 `docs/product/mvp_freeze_manifest.json`。

本次冻结不是新后端版本发布，不创建 tag 或 GitHub Release。v0.32.0 继续作为后端冻结
基线，v0.33.0 继续作为当前后端维护版本。

## 发布前验收

固定 Rust 1.95.0 后执行：

```bash
scripts/ai/check_mvp_freeze_baseline.sh
cargo build -p nautilus-cli --bin nautilus --bin ntpro-node
NTPRO_MVP_ACCEPTANCE_EVIDENCE_DIR=/tmp/ntpro-mvp-acceptance \
  node scripts/ai/test_mvp_acceptance.mjs
NTPRO_MVP_FAULT_EVIDENCE_DIR=/tmp/ntpro-mvp-fault-matrix \
  node scripts/ai/test_mvp_fault_matrix.mjs
```

浏览器验收必须在 `1440x1000` 与 `390x844` 两个视口分别覆盖机构工作台和控制中心。
GitHub PR 必须同时通过 `Rust Cutover Smoke` 的四类 MVP artifact、六组
`Backend Performance` same-runner 对比、独立审查和其他 required checks。

## 本地启动

```bash
cargo run -p nautilus-cli --bin nautilus -- \
  mvp serve \
  --config configs/nodes/btc-ema-shadow.toml \
  --workspace /tmp/ntpro-mvp-workspace \
  --bind 127.0.0.1:8080 \
  --ntpro-node-bin target/debug/ntpro-node
```

只使用启动日志生成的一次性角色入口。不得把 bootstrap token 写入文档、截图、工件或
共享命令。机构入口只能进入 `/institution-workbench`；operator 入口只能进入
`/control-center`。

## 停止条件

出现以下任一情况立即停止候选，不得解释为可降级上线：

- 冻结守卫、确定性闭环、故障矩阵、任一浏览器验收或性能 merge-authority job 失败；
- 节点身份、代际、status/metrics 工件不一致或陈旧；
- required-false 字段缺失或变成 true；
- HTTP 200 或进程存活被错误解释为技术健康；
- 出现外部 Venue、公网请求、真实订单、订单变更、自动 retry、remediation 或 recovery；
- artifact 中出现 access token、cookie 或其他 secret。

## 人工回滚

1. 由 operator 通过控制中心执行一次明确确认的 `stop`，或在终端向 MVP server 发送
   `SIGINT`；不得使用自动 restart/retry。
2. 确认 Supervisor registry 中节点 process 与 lifecycle 都是 `stopped`，共享状态的
   `trading_readiness` 是 `blocked`，所有交易边界仍为 false。
3. 停止当前进程后，将代码回退到最后一个已审查、冻结守卫通过的提交。不得重写
   v0.32.0/v0.33.0 tag，也不得修改历史 release 工件。
4. 在新的空 workspace 中重新执行完整发布前验收。不得复用故障场景遗留的 PID、角色
   cookie、status/metrics 或 registry 工件。
5. 如果故障涉及合同或冻结源文件，必须建立独立 GitHub issue、风险说明、回滚证据和
   manifest 更新；禁止直接在冻结基线上做未登记修改。

## 冻结后的变更入口

多节点、多账户、多策略、多 Venue、生产 IAM、实盘下单或产品级桌面终端都不属于本
MVP。任何后续能力必须独立立项，不得继承本基线中保持关闭的能力。
