# 策略工作台前端产品架构

Date: 2026-08-05
Executor: Codex
Status: implementation contract

## 1. 文档目的

本文档定义 NTPRO 策略工作台从页面框架进入正式产品开发时采用的前端架构、产品资源、
页面路由、数据边界、构建方式、测试要求和交付顺序。

当前 `/strategy-workbench` 已采用 React、TypeScript、Vite、TanStack Router、TanStack
Query、CSS Modules、Vitest 与 Playwright，并由 Rust/Axum 同源提供生产静态资源。
`GET /api/mvp/v1/status` 继续提供辅助技术状态；策略总览与 Run 详情通过生成的
`productApi` 消费 Strategy、StrategyVersion 和 Run 只读产品资源。

## 2. 产品目标与用户

策略工作台是 NTPRO 当前唯一主产品入口，默认用户是策略研发与运行人员。用户围绕同一个
不可变 `StrategyVersion` 创建和管理三种彼此独立的运行：

```text
Strategy
  -> StrategyVersion（不可变）
       -> Backtest Run
       -> Demo Run（代码中的 Sandbox）
       -> Live Run
```

三种 Run 共享策略逻辑、参数结构、订单语义、风险指标与证据格式，但分别绑定自己的数据、
环境、账户、Venue、适配器、权限、状态和结果。Backtest 或 Demo 通过不能自动授予 Live
权限。

Supervisor、node、Axum、文件路径和进程控制是技术支撑，不主导产品导航。系统状态只作为
辅助诊断入口。

## 3. 当前事实与能力边界

当前已经具备：

- Rust `BacktestEngine`；
- Sandbox 与 Live 共用语义的 `LiveNodeBuilder`；
- 单 Supervisor + 单 Sandbox node 的 Demo MVP；
- 版本化只读共享状态 API；
- 策略工作台页面框架与响应式浏览器验证。
- 独立前端工程、组件体系和类型化路由；
- 稳定、强校验的 `Strategy / StrategyVersion / Run` 只读产品 API；
- 策略总览和 Run 详情首个只读产品纵向切片。

当前尚未具备：

- 页面可操作的 Backtest 产品闭环；
- 页面可操作的 Demo 产品闭环；
- 真实账户、真实 Venue、真实订单和 Live 操作权限。

在对应能力完成独立验收前，前端不得展示可用的 submit、cancel、replace、amend、flatten、
retry 或 remediation 控件，也不得用 fixture、HTTP 200 或进程存活冒充产品能力。

## 4. 技术选型

### 4.1 前端应用

```text
React + TypeScript
Vite
TanStack Router
TanStack Query
CSS Modules + CSS design tokens
Lucide React
Vitest + React Testing Library
Mock Service Worker
Playwright
```

选择理由：

- React 将页面拆成可复用组件，适合策略上下文、密集表格、状态面板和活动 Dock；
- TypeScript 约束 API 数据、路由参数、模式状态和错误分支；
- Vite 提供开发服务器，并构建可由 Axum 提供的静态资源；
- TanStack Router 管理策略、版本、Run 与比较页面的类型化 URL 状态；
- TanStack Query 管理服务端状态、缓存、刷新、失效与请求错误；
- CSS Modules 保留当前终端风格，CSS design tokens 固定颜色、间距、字号和密度；
- Lucide 提供一致的界面图标，不在业务代码中维护手写 SVG；
- Vitest、Testing Library、MSW 与 Playwright 分别覆盖组件、交互、API 边界和真实浏览器流程。

一期不引入 Next.js、服务端渲染、Redux、Zustand 或大型通用组件主题。NTPRO 已有 Axum
服务端，同源单页应用不需要额外 Node.js 生产服务；客户端状态先使用 URL 与 React 局部
状态，服务端状态由 TanStack Query 管理。

官方依据：

- React TypeScript：https://react.dev/learn/typescript
- Vite Guide：https://vite.dev/guide/
- TanStack Router：https://tanstack.com/router/latest
- TanStack Query：https://tanstack.com/query/latest/docs/framework/react/overview
- Playwright：https://playwright.dev/docs/intro

### 4.2 构建和运行边界

前端源码放置在 `apps/strategy-workbench/`。Node.js 和 npm 只用于开发、检查、测试和构建，
不得成为生产运行时依赖。

```text
apps/strategy-workbench/src
  -> npm ci / npm run build
  -> apps/strategy-workbench/dist
  -> Rust/Axum 提供 index.html 与带 hash 的静态资源
```

生产进程仍只有 Rust/Axum。浏览器与 API 同源，默认不增加 CORS 配置。Axum 应将
`/strategy-workbench/*` 的前端路由回退到同一 `index.html`，同时保留 API 路由的明确
404/405 语义。

开发环境允许 Vite 代理 `/api` 到本地 Axum。生产构建不得提交 `node_modules/`、测试缓存
或开发服务器状态；只能提交源码、lockfile 和经治理允许的构建接入代码。

## 5. 目录结构

```text
apps/strategy-workbench/
  package.json
  package-lock.json
  tsconfig.json
  vite.config.ts
  index.html
  src/
    app/             路由、Provider、错误边界
    layouts/         左侧栏、顶部栏、主画布、详情栏、活动 Dock、状态栏
    pages/           总览、策略、Backtest、Demo、Live、运行、数据、风险、系统状态
    features/        按业务能力组织的组件、查询与操作
    components/      无业务归属的通用界面组件
    api/             生成类型、请求客户端、错误转换
    styles/          全局变量、基础样式、密度和响应式规则
    test/            MSW handlers、fixtures 与测试辅助函数
  tests/e2e/         Playwright 用户流程
```

页面文件不直接调用 `fetch`，请求集中在 `api/` 与 feature query 中。通用组件不得依赖具体
Strategy 或 Run 数据结构。

## 6. 产品资源与 API 前置合同

前端业务页必须建立在以下稳定资源之上：

### Strategy

- `strategy_id`：稳定身份；
- 名称、描述、负责人和生命周期状态；
- 当前默认版本引用；
- 创建时间与更新时间。

### StrategyVersion

- `strategy_version_id` 与内容 hash；
- 所属 `strategy_id`；
- 不可变代码引用、参数 schema、数据要求和风险配置；
- 创建来源、时间与状态；
- 已创建后禁止原地修改。

### Run

- `run_id` 与 `strategy_version_id`；
- `environment`：`backtest | sandbox | live`；
- 数据、配置、适配器、账户和 Venue 引用；
- `queued | starting | running | stopping | completed | failed | cancelled | blocked`；
- 时间、结果、风险、来源和错误合同。

首批只读产品 API：

```text
GET /api/product/v1/strategies
GET /api/product/v1/strategies/{strategy_id}
GET /api/product/v1/strategies/{strategy_id}/versions
GET /api/product/v1/strategies/{strategy_id}/versions/{strategy_version_id}
GET /api/product/v1/runs
GET /api/product/v1/runs/{run_id}
GET /api/product/v1/runs/{run_id}/metrics
```

其中 metrics 只服务已完成且结果可用的 Backtest Run。以下资源仍属于 S1 后续任务，不得
在首个切片中冒充已交付：

```text
GET /api/product/v1/runs/{run_id}/positions
GET /api/product/v1/runs/{run_id}/orders
GET /api/product/v1/runs/{run_id}/fills
GET /api/product/v1/runs/{run_id}/events
```

列表合同必须定义游标分页、排序和筛选。错误合同必须提供稳定错误码、用户可读摘要、请求
关联 ID 和是否可重试，不向浏览器暴露原始内部错误。TypeScript 类型应从后端 schema 或
OpenAPI 生成，禁止长期手写两套同名合同。

现有 `/api/mvp/v1/status` 继续服务 Demo 技术状态和系统诊断，但不能代替 Strategy、
StrategyVersion 或 Run 产品 API。

## 7. 页面信息架构

### 固定框架

- 左侧导航：总览、策略、Backtest、Demo、Live、运行、数据、风险、系统状态；
- 顶部上下文：Strategy、StrategyVersion、Environment、账户、Venue；
- 主画布：当前页面主要任务；
- 右侧栏：当前选中对象的来源、状态、边界与诊断；
- 底部 Dock：持仓、活动、成交、日志；
- 状态栏：数据、账户、Venue、风险、node 与更新时间。

### 首个纵向切片

第一轮只完整交付“策略总览 + Run 详情”：

- 策略列表与策略详情；
- 不可变版本列表与当前版本；
- Backtest、Demo、Live Run 汇总；
- Run 状态、来源、环境和风险阻断；
- 从 Run 进入指标、持仓、订单、成交与事件详情；
- 加载、空数据、无权限、陈旧、合同错误和服务不可用状态。

Backtest、Demo 和 Live 后续分别扩展。数据、风险和系统状态一期先作为 Run 上下文的子视图，
不先建设三套缺少真实合同的独立空页面。

## 8. 状态和交互规则

- 可分享的选择状态写入 URL：策略、版本、Run、模式、页签、筛选与时间范围；
- API 服务端状态由 TanStack Query 管理，不复制到全局 store；
- 详情抽屉、Dock 展开状态等短期交互使用 React 局部状态；
- 切换 StrategyVersion 时必须清理不属于新版本的 Run 与旧详情；
- schema、身份、来源或边界校验失败时清空旧数据并 fail closed；
- Demo 和 Live 的刷新频率按页面可见性调整，不在隐藏标签页持续高频轮询；
- 写操作必须有独立 mutation 合同、幂等键、确认结果和审计引用，不能由按钮直接拼接请求。

## 9. 视觉与可用性规则

- 沿用 SWB-001 已确认的深色中性底色和紧凑机构交易终端风格；
- 不使用大面积营销式卡片、渐变背景、装饰光球或超大标题；
- 表格、工具栏、状态栏和 Dock 使用稳定尺寸，动态内容不得造成布局跳动；
- 桌面优先支持高密度扫描，同时保证 390px 移动视口无页面级横向溢出；
- 图标按钮使用 Lucide 图标和 tooltip；文本按钮只用于明确命令；
- 红色表示阻断或风险，绿色只表示已验证正常，不用颜色作为唯一信息载体；
- 空状态、加载、陈旧、失败、阻断和无权限必须分别呈现。

## 10. 验证与 CI

前端任务至少执行：

```text
npm ci
npm run format:check
npm run lint
npm run typecheck
npm run test
npm run build
npm run test:e2e
```

Rust/Axum 集成任务还必须验证：

- 静态资源 content type 与缓存头；
- SPA 路由回退和 API 404/405 不被吞掉；
- 机构访问边界与 URL token 清理；
- 无前端源码或 Node.js 生产运行时依赖；
- 1440x1000 和 390x844 浏览器截图、非空像素和无重叠；
- 现有 Rust fast gate、冻结基线和交易控制禁用边界继续通过。

CI 按路径分类：纯前端变更执行前端 lint、类型、单测、构建和浏览器 smoke；前端与 Axum
集成变更再执行目标 Rust 测试。不得让每个纯前端改动无条件重复完整 Rust workspace gate。

## 11. 迁移与交付顺序

```text
FEA-001 前端架构文档
  -> FEF-001 前端工程底座
  -> FEI-001 Axum 静态资源接入与旧页面迁移
  -> S0-API-001 产品资源只读合同
  -> SWB-002 策略总览与 Run 详情
  -> S1 Backtest 产品化
  -> S2 Demo 产品化
  -> S3 Live 独立准入与产品能力
  -> S4 三模式比较、验收与冻结
```

`FEI-001` 依赖 `FEF-001`。`FEF-001` 与 `S0-API-001` 可以并行，但 `SWB-002` 必须同时
依赖 `FEI-001` 与 `S0-API-001`。旧 Rust 内嵌页面在新应用达到同等只读、响应式、错误和
边界验收前继续保留；由 `FEI-001` 定义生产静态资源打包、Axum SPA 回退和旧页面迁移，
禁止在底座任务中提前移除回退能力或提交临时 `dist/`。

## 12. FEF-001 退出条件

- `apps/strategy-workbench/` 可开发、测试和生产构建；
- 当前页面框架已组件化，中文信息架构与视觉风格不倒退；
- Vite 可生成生产静态资源，`dist/` 保持未跟踪；
- typed API client、错误边界、查询 Provider、测试 fixture 与 Playwright 基础可用；
- `/api/mvp/v1/status` 仅作为当前只读状态桥接；
- 不声称 Strategy、StrategyVersion、Run、Backtest、Demo 或 Live 产品页已完成；
- 不新增真实交易能力或生产 Node.js 运行时。

## 13. FEI-001 退出条件

- 生产静态资源打包方式不要求 Cargo 隐式运行 npm；
- Axum 提供带 hash 的前端资源和 `/strategy-workbench/*` SPA 回退；
- API 404/405 不被 SPA 回退吞掉，访问与 token 清理边界保持成立；
- 新应用达到旧页面的只读、错误、桌面与移动验收；
- 旧 Rust 内嵌 HTML、CSS 与 JavaScript 只在等价验收后删除。
