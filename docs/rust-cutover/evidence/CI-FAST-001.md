# CI-FAST-001 - Pull Request 验证并行化证据

Date: 2026-08-13
Executor: Codex
GitHub issue: #1304
Status: LOCAL_VALIDATION_PASSED_REVIEW_REQUIRED

## 修改前基线

- PR #1303 的 Rust Cutover Smoke run `31696445012` 约 10 分 59 秒；
- 同一 PR 的 Backend Performance run `31696445049` 约 13 分 13 秒；
- Smoke 中 workspace check、Clippy、产品 lint、CLI tests、MVP 与浏览器检查串行执行；
- Backend Performance 对所有 `crates/**` PR 启动六个 baseline/candidate workload；
- classifier 对任意 `crates/**` 和 Cargo 变更启动 MVP acceptance 与 fault matrix。

## 实现合同

- Backend Performance 保留定时与手动入口，删除 PR 入口；
- classifier 只让冻结源和明确 MVP 文件进入 MVP 总验收；
- `smoke-core`、`rust-lint` 和 `rust-tests` 在分类后并行执行；
- 最终 `smoke` 在 `always()` 下通过本地可测的 fail-closed 脚本聚合所有必要 lane，
  拒绝失败、取消、错误跳过或无效分类；
- Clippy 提供 workspace 编译与 lint 覆盖，不再提前重复执行 `cargo check`。

## 验证

- classifier 与聚合器正负向自测：PASS，36 cases；
- Rust 1.95 工具链绑定：PASS，6 个 workflow bindings；
- 五个 workflow YAML：PASS；
- 本机未安装 `actionlint`，GitHub Actions 表达式与 job 调度仍需以 hosted checks 为最终
  证据；
- `scripts/ai/verify_fast.sh`：PASS；
- `cargo test -p ntpro-governance`：PASS，35 tests；
- Backend Performance contract：PASS，6 workloads；
- MVP freeze：PASS，13 frozen sources / 19 disabled boundaries；
- backend freeze、zero-Python、backend hygiene、Rust-only 和 docs/examples：PASS；
- backend runtime risk inventory：信号数、文件数和 ownership 计数均未变化，只更新由
  治理 trigger 字符串变化导致的 canonical hash；
- current governance：PASS；v0.33 maintenance release stage 在模拟 hosted
  `pull_request` 事件上下文下 PASS；

## Hosted 首轮冷缓存证据

- PR #1305 / Rust Cutover Smoke run `31702533792`：success；
- `changes` 23 秒后，`smoke-core`、`rust-lint`、`rust-tests` 同时开始，证明并行调度成立；
- `rust-tests` 6 分 54 秒，`smoke-core` 11 分 50 秒，`rust-lint` 14 分 15 秒，最终
  required `smoke` 9 秒并成功聚合；
- Backend Performance 未被该普通 PR 触发；security run `31702533727` 全部成功；
- 新 job 的首轮 cache key 无历史条目：workspace Clippy 冷编译 8 分 36 秒，第二组
  warning-only product lint 冷编译 5 分 07 秒；本轮用于建立缓存，不能作为最终提速结论；
- 下一 head 将复用完全相同的 Cargo lockfile、Rust 1.95 和 job cache key，记录热缓存耗时
  后再判断 required gate 是否达到提速目标。

## Hosted 热缓存验收

- PR #1305 / Rust Cutover Smoke run `31703895576`：success；
- `rust-tests` 2 分 11 秒、`rust-lint` 4 分 34 秒、`smoke-core` 6 分 50 秒，最终
  required `smoke` 8 秒并成功聚合；
- 对比 PR #1303 修改前 run `31696445012` 的约 10 分 59 秒，本次修改 workflow 自身、
  因而强制执行完整 MVP/门户验收的最坏路径减少约 4 分 09 秒，约 38%；
- 普通非冻结 Rust 变更不会再启动 MVP acceptance/fault，且三条 lane 并行，预计由约
  4 分 34 秒的 lint lane 决定等待时间；实际长期分位数应在后续普通产品 PR 中持续观察；
- security run `31703895709` 全部成功；该 head 仍未触发 Backend Performance；
- branch protection `strict=true`，唯一 required context 仍为 `smoke`；
- Hosted 结构、并行调度、缓存复用和 required 聚合均验证通过，等待独立 review。

## 行为影响

只改变 CI 调度和等待时间，不修改产品运行时、公开 API、交易语义、真实 Live 权限、
冻结源或发布边界。

## 回滚

回退本任务提交即可；性能 workflow 的六个 workload 和所有验证脚本均仍保留。
