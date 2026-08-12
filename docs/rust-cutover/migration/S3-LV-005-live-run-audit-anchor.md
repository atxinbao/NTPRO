# S3-LV-005 Live Run 外部审计锚点迁移说明

Date: 2026-08-12
Executor: Codex

## 破坏性变更

Live Run 本地状态从 `state.v1` / `state_head.v1` 升级到 v2，并要求每个 revision 同时存在有效的
`anchor-receipt-<revision>.json`。旧候选没有外部历史证明，系统不会自动补签或把当前文件冒充
历史 WORM 证据，因此旧 v1 候选会 fail closed。

升级前必须确认没有活动 Live Run 候选，将旧 `artifacts/live-runs/` 和
`artifacts/live-run-state-commits/` 作为只读历史归档移出活动工作区，再配置外部锚点并创建新
候选。禁止删除远端锚点或重置 namespace 来绕过 revision 冲突。

## 外部服务要求

部署方提供支持 HTTPS、Bearer 认证、namespace 级 workspace compare-and-append、全局单调
workspace revision、latest 读取和 Ed25519 签名回执的独立服务。服务必须位于 NTPRO 工作区
故障域之外，并由实际不可变存储、WORM 日志或具有单调 CAS 保证的控制面支撑；普通共享目录、
同机 SQLite 或可覆盖对象不能作为生产证明。

## 故障处理

- 远端不可用或响应无效：保持候选和交易能力阻断，不自动重试；
- 远端 revision 高于本地：本地可能发生完整快照恢复或提交后崩溃，隔离工作区并按远端回执恢复；
- 本地高于远端：视为远端数据丢失或错误 namespace，禁止补写覆盖；
- key id 或签名不匹配：按审计密钥事件处理，禁止临时忽略验证；
- token 轮换只更新部署 secret，不改历史回执；签名公钥轮换必须建立新的独立迁移任务。

## 回滚

代码回滚只能恢复“候选功能不可用”的版本，不能撤销或覆盖已经写入的远端回执。回滚期间保持
Live Runtime、行情和真实订单全部关闭；重新启用候选前必须重新完成锚点一致性检查。
