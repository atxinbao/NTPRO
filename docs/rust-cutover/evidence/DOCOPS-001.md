# DOCOPS-001 - NTPRO 系统运行与运营操作说明书证据

Date: 2026-08-05
Executor: Codex
GitHub issue: #1239
GitHub PR: PENDING
Status: DONE_ON_MERGE

## 权威事实

- `docs/product/mvp_freeze_manifest.json` 固定一个 Supervisor、一个 node、一个策略实例、
  一个账户、一个 Venue 和 sandbox 环境，M0-M4 已完成或冻结；
- 当前产品入口是 `nautilus mvp serve`，机构工作台为
  `/institution-workbench`，控制中心为 `/control-center`；
- 两个门户消费 `GET /api/mvp/v1/status` 等共享只读投影；
- operator 动作仅有单节点 sandbox `start` 和 `stop`，机构工作台保持只读；
- 19 个真实交易、外部连接、自动恢复、多节点和错误推断边界继续为 false；
- `crates/cli/src/opt.rs` 固定默认 loopback 地址、节点 ID、心跳和一小时最长运行时间；
- `crates/cli/src/mvp.rs` 负责启动节点、刷新状态合同，并在 Dashboard、刷新或 Ctrl-C
  退出时停止节点。

## 变更边界

本任务只新增人类操作说明和治理证据，并修正 README 的陈旧 MVP 状态。它不修改任何
冻结源、manifest、运行时、API、workflow、tag 或 release，也不声明当前已具备未来的
远程多节点能力。

## 验证

- 文档事实与机器冻结合同、roadmap、发布/回滚说明和 CLI 参数交叉核对；
- `scripts/ai/check_docs_examples_governance.sh`：PASS，134 个 Markdown 文件和 315 个
  本地链接通过；
- `scripts/ai/check_mvp_freeze_baseline.sh`：PASS，19 个任务、5 个阶段、19 个关闭边界和
  13 个冻结源成立；
- `scripts/ai/check_rust_toolchain_pin.sh`：PASS，Rust 1.95.0、16 个直接 Cargo 脚本和
  4 个 workflow binding 成立；
- `scripts/ai/check_backend_freeze_baseline.sh`：PASS，20 个负向用例成立；
- `scripts/ai/verify_release.sh current-governance backend-freeze-baseline`：PASS；
- `git diff --check`：PASS；
- hosted checks 和 PR merge 事实由 GitHub 提供。
