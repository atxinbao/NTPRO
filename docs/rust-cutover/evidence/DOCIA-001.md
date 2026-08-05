# DOCIA-001 - project.html 产品说明与技术说明分层证据

Date: 2026-08-05
Executor: Codex
GitHub issue: #1241
GitHub PR: #1242
Status: DONE_ON_MERGE

## 权威事实

- `docs/product/mvp_freeze_manifest.json` 固定单 Supervisor、单 node、单策略实例、单账户、
  单 Venue 的 sandbox MVP，M0-M4 已完成或冻结；
- `docs/product/roadmap.md` 和 PR #1236/#1238 已证明 MVP-013 与 M4 技术/文档收口完成；
- `docs/product/ntpro_system_operations_manual.md` 已解释当前运行逻辑和运营边界；
- 机构工作台面向业务用户，控制中心面向平台控制人员，共享控制平面不是第三个产品；
- 当前不具备真实订单、外部 Venue、多节点生产编排、自动恢复或产品级实盘终端。

## 变更边界

本任务只重组项目说明页面的信息层级和交互，不修改运行时代码、API、冻结源、workflow、
tag 或 release。页面中的商业运营流程是目标运营模型，不代表当前 MVP 已具备商业生产能力。

## 验证

- 产品/技术入口与 hash 导航：PASS，产品页显示 3 个产品章节，技术页显示 7 个技术
  章节，入口、导航和 URL hash 始终一致；
- 桌面 `1440x1000` 浏览器验证：PASS，两个说明入口、左侧导航和主体无重叠或横向
  溢出；
- 移动 `390x844` 浏览器验证：PASS，页面宽度等于视口宽度，顶部切换和横向章节导航
  可用；
- 页面截图和控制台错误检查：PASS，产品/技术桌面与移动截图已检查，console errors
  为 0；
- 17 个本地页面/文档/源码链接：PASS，HTTP 状态均为 200；
- 打印模式：PASS，10/10 个产品与技术章节全部可见；
- `scripts/ai/check_docs_examples_governance.sh`：PASS，134 个 Markdown 文件和 315 个
  本地链接通过；
- `scripts/ai/check_mvp_freeze_baseline.sh`：PASS，19 个关闭边界和 13 个冻结源成立；
- `git diff --check`：PASS；
- hosted checks 和 PR merge 事实由 GitHub 提供。
