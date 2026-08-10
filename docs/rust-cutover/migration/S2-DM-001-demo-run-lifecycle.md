# S2-DM-001 Demo Run 生命周期迁移说明

Date: 2026-08-10
Executor: Codex

## 行为变化

默认节点配置不再预置一个声称处于 `running` 的 Sandbox Run。Demo Run 必须由策略工作台
或 Product API 显式创建，并由实际 Supervisor start/stop 结果决定生命周期。

`nautilus mvp serve` 启动后只准备服务和注册表，node 保持 stopped；不再由 MVP 入口预启动。
服务退出时会先停止并终态化当前 ownership 对应的活动 Demo，再关闭其余运行资源。

## 兼容边界

- Backtest Run、比较与复现合同保持不变；
- Live Run 继续是禁用占位资源；
- 旧的静态 Sandbox Run 不再作为可信运行事实，用户需要重新创建 Demo Run；
- 不开放真实订单或外部 Venue。

## 使用方式

机构用户进入策略工作台 Demo 页面，确认当前 StrategyVersion、Supervisor 节点、Sandbox
账户和 Venue 后创建 Demo Run。创建只登记不可变运行身份，不会自动启动节点；进入 Run 详情
后，用户可显式启动或停止。客户端不会自动重复生命周期 POST。

同一 Sandbox node 在任一时刻只承载一个活动 Demo Run。停止或失败的 Run 保留为历史记录，
不能再次启动；用户可以基于同一 StrategyVersion 创建新的 Demo Run。新 Run 复用 node 时不会
改写旧 Run 的终态。

同一 workspace 重启不会恢复或自动启动旧 node。无运行工件的 NotStarted 准备态、已锚定的
stopped/failed Run 及其只读策略来源可长期读取；运行中的过期来源、未锚定终态、边界漂移
或产物篡改仍会 fail closed。

重启 prepare 会删除上一次已停止节点留下的 status、metrics 和 pid 运行快照，再以 Missing
状态发布新的准备态合同；日志、不可变 Demo manifest、terminal state 和 registry 哈希锚点
继续保留。运行中的 status contract 会随显式 start/stop 刷新，超过配置 freshness 窗口时
Product API 返回 stale，不会把旧合同声明为 fresh。

活动 Demo Run 会在 Supervisor registry 中取得该 node 的独占 ownership。此时控制中心和
`nautilus supervisor` 通用启停命令不会绕过策略工作台的 Demo 生命周期；必须从对应 Demo Run
执行 start/stop。自然停止或异常退出会先形成由 Supervisor 哈希锚定的终态，节点修复后才可
进入下一次 Demo 创建流程。

API 消费方必须按 OpenAPI 生成合同迁移：Run 新增可空 `runtime`，生命周期新增 `paused`，并
新增 Demo 创建与动作响应。旧的静态 Sandbox Run 配置应删除，不得继续冒充实际运行状态。
