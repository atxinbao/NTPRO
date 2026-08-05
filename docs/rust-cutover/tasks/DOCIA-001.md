# DOCIA-001 - project.html 产品说明与技术说明分层

Date: 2026-08-05
Executor: Codex
GitHub issue: #1241
GitHub PR: #1242
Risk: low
Owner role: Docs & Developer Experience Agent
Review role: Verification & Release Gatekeeper
Status: DONE_ON_MERGE

## 目标

将 `project.html` 从产品、技术和治理内容混合的单层文档，重组为“产品说明”和“技术
说明”两个独立入口。产品说明面向所有人并使用业务语言，技术说明面向研发、架构、
运维和审计人员并保留专业合同与证据。

## 范围

- 保留现有浅色视觉风格、顶部栏目、左侧菜单和主体布局；
- 新增产品/技术分段切换和各自独立的左侧导航；
- 产品说明新增商业运营闭环与双门户分工；
- 技术说明承接详细信息架构、后端架构、技术缺口、目标框架、Roadmap、验收与证据；
- 支持 URL hash 直接打开对应部分，打印时输出完整两部分；
- 修正 MVP-013/M4 已完成冻结的陈旧状态。

## 非目标

不修改 Rust 运行时、API、workflow、冻结 manifest、release 或交易能力；不改变已确认的
页面底色和整体视觉方向；不把未来多节点、真实交易或商业生产能力声明为当前能力。

## 验收

- 产品和技术说明均可独立进入、导航和打印；
- 桌面与移动视口无重叠、横向溢出或空白主内容；
- 产品说明不要求理解代码、crate、API 或发布治理；
- 技术说明保留架构、边界、验收和来源；
- docs governance、MVP freeze、HTML 交互自测和 `git diff --check` 通过；
- PR 合并后 `DONE_ON_MERGE` 生效并关闭 Issue #1241。
