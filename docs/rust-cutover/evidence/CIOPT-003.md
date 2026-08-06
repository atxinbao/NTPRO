# CIOPT-003 - MVP 浏览器验收退出竞态修复证据

Date: 2026-08-06
Executor: Codex
GitHub issue: #1264
GitHub PR: #1265
Status: REVIEW_REQUIRED

## 失败证据

- PR #1263 的 Rust Cutover Smoke run `31060491696` attempt 1 在浏览器业务断言完成后失败；
- 失败信息为 `MVP server did not stop within 5000 ms and required SIGKILL`；
- 失败 artifact 的 node stdout/stderr 为空，说明外层在内层停止窗口结束前后强杀进程；
- 同一 commit 本机连续三次通过，确认缺陷依赖 hosted runner 调度时序；
- attempt 2 全部通过，产品合同本身没有失败。

## 修复合同

- node 内层停止超时显式设为 5000 ms；
- 测试进程外层优雅退出等待设为 15000 ms；
- 自测要求外层严格大于内层，并拒绝相等、小于和非正 node 超时；
- Supervisor 正常停止写 `shutdown_escalated=false`；进入 TERM/KILL escalation 时写 true，
  完成状态收口后仍返回错误；
- 浏览器启动真实 Node.js 异常子进程，使其忽略 SIGINT，并断言共享停止函数进入 SIGKILL
  后抛错；
- 成功必须同时出现服务停止、node 最终停止、`shutdown_escalated=false` 和禁用交易边界证据。

## 验证

- `node --check scripts/ai/test_institution_workbench_browser.mjs`：PASS；
- CLI 全量单元测试：605/605 PASS；
- Clippy `--all-targets --all-features -D warnings`：PASS；
- Supervisor stop 定向测试：4/4 PASS，覆盖正常停止、暂停后停止、缺少 Stopped artifact 的
  escalation，以及有 Stopped artifact 仍拒绝 escalation；
- 重新构建 CLI binaries 后，真实 MVP + Chrome 浏览器验收连续运行 3 次：3/3 PASS，输出
  `shutdown_timeout_contract_selftest=1`、`shutdown_escalation_selftest=1`、
  `shutdown_escalated=0` 与 `graceful_shutdown=1`；
- `scripts/ai/check_mvp_freeze_baseline.sh`：PASS，19 个关闭边界、13 个冻结源；
- `scripts/ci/test-ci-change-classifier.sh`：PASS，21 cases；
- `scripts/ai/check_backend_freeze_baseline.sh`：PASS，v0.32.0 的 27 个关闭边界和 4 个
  冻结源未变化；
- `scripts/ai/check_rust_only_runtime.sh`：PASS；
- backend runtime risk inventory：29528 signals / 1224 files，新增 9 个均属于 inline tests；
- risk inventory self-test：8/8 PASS；
- docs/examples governance：135 个 Markdown、315 个本地链接；
- `git diff --check`：PASS；
- 独立复审和 hosted checks：等待 PR。

## 行为影响

修复测试编排时序，并让 Supervisor 的强制终止从“可能返回成功”改为明确错误。没有修改
产品 API、浏览器页面、真实 Live 权限或交易边界。

## 回滚

回退本任务提交并恢复冻结 manifest hash；不涉及数据、schema、部署或 release 回滚。
