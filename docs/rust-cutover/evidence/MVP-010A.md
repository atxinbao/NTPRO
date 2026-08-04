# MVP-010A - M3 生命周期动作合并后状态收口证据

Date: 2026-08-04
Executor: Codex
GitHub issue: #1227
Status: LOCAL VALIDATION PASSED

## 远端事实

- MVP-010 issue：#1225 / closed at `2026-08-04T11:06:41Z`；
- MVP-010 PR：#1226 / merged at `2026-08-04T11:06:40Z`；
- final head：`a41a9b722a289765047562f2100640a7642d60fa`；
- merge SHA：`97fdd88a82fae342b4a909e1b3f25e8d9feef20e`；
- 独立 Verification & Release Gatekeeper：exact final head `REVIEW_APPROVED`；
- Rust Cutover Smoke：run `30902101122` / success；
- Backend Performance：run `30902101201` / 6/6 jobs success；
- security-audit：run `30902101202` / success；
- hosted checks：14/14 success；
- `control-center-browser-1226/result.json`：pass，artifact id `8889719905`，digest
  `sha256:2d3fbc07e4c7df4101af527867e2f10f8fcca8981b3467e95ddece4f3b5133a5`；
- `institution-workbench-browser-1226/result.json`：pass，artifact id `8889719146`，digest
  `sha256:9df96384eb5f164edfd1effff5c57e95bdcdf6aeb315e2a5bf76c73e036bb096`；
- hosted artifact 令牌扫描：未发现真实 `access_token` 值；
- 桌面与 390x844 截图：无重叠，机构工作台无 lifecycle action 控件。

## 状态结论

MVP-010 已把版本化、operator-only、POST-only 的 start/stop 产品合同接入唯一的本地
sandbox 节点，并完成 scope、并发、目标最终状态、错误信封和脱敏边界验证。机构工作台
继续保持只读，两个门户引用同一运行实例与事件证据。

M3 退出条件已满足：机构用户可以从机构工作台回答策略与业务状态，运维用户可以从控制
中心回答节点状态与技术根因，并通过事件合同互相追踪。下一阶段是 M4 MVP 验收与冻结。

## 保持关闭的能力

M3 完成不表示真实交易、订单提交/变更、外部 Venue、多节点编排、生产 IAM、自动重试或
自动补救已授权。生命周期动作仍只适用于本地单 Supervisor + 单 sandbox 节点。

## 本地验证

使用仓库固定 Rust `1.95.0` 完成以下验证：

- `scripts/ai/check_docs_examples_governance.sh`：pass，`markdown_files=132`、
  `local_links=311`、`image_links=20`；
- `scripts/ai/check_rust_only_runtime.sh`：pass；
- `scripts/ai/check_backend_freeze_baseline.sh`：pass，冻结 tag、commit、27 个边界和
  4 个 source hash 均匹配；
- `scripts/ai/verify_release.sh current-governance backend-freeze-baseline`：pass，包含
  current release、backend hygiene、ignored tests、runtime risk inventory、control plane
  retirement 与 historical release retirement；
- `git diff --check`：pass；
- `project.html` 桌面与 `390x844` 浏览器截图：pass，导航、状态标签和正文无重叠。
