# NTPRO Rust-only v0.33.0

Status: RELEASE GATE READY
Tag: `ntpro-rust-only-v0.33.0`
Release name: `NTPRO Rust-only v0.33.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.33.0`
Base release: `ntpro-rust-only-v0.32.0`
Date: 2026-07-21
Executor: Codex

v0.33.0 is a separately scoped, maintenance-only release over the frozen
v0.32.0 Backend Production Closeout baseline. It improves backend measurement,
hosted regression detection, CLI module ownership, checked runtime error
boundaries, dependency surfaces, and one benchmark-proven rate-limiter path.
It does not reopen the backend mainline or rewrite the v0.32.0 baseline.

中文摘要：v0.33.0 只发布已经独立立项并完成审查的后端维护与性能优化。它建立了
可复现基准和 hosted 回归检查，整理 CLI 模块边界，加固插值、收益率曲线、cache 和
回测错误边界，移除无效依赖，并优化默认 rate limiter 路径。它不代表后端重新进入
功能开发，不开放实盘下单、订单修改、adapter send、live exchange、自动重试/
补救/恢复，也不增加 Dashboard、Admin Workbench 或 Trader Terminal 交易控件。

## Included Tasks

- `BPO-001` / #1120 / PR #1134 - reproducible performance and resource baseline.
- `BPO-002` / #1121 / PR #1135 - hosted benchmark and regression workflow.
- `BPO-003` / #1122 / PR #1136 - behavior-preserving CLI module decomposition.
- `BPO-004` / #1123 / PR #1137 - checked runtime error and panic boundaries.
- `BPO-005` / #1124 / PR #1138 - feature, dependency, build, and binary cleanup.
- `BPO-006` / #1125 / PR #1139 - measured default rate-limiter optimization.
- `BPO-007` / #1126 / PR #1140 - release gates, strict provenance, publication, and closeout.

Exact issue set = #1120-#1126
Exact PR set = #1134-#1140
Exact issue count = 7
Exact PR count = 7
Milestone = v0.33.0-backend-maintenance

## Performance Evidence

The fixed contract covers six core, model, data, execution, live, and network
workloads. Five stable workloads have merge authority; the noisy execution
lookup remains informational. BPO-006 improved the selected default
rate-limiter path by 12.422% in the stable local confirmation and 22.067% in
hosted run `29758547202`. Correctness, concurrency, full network tests, Clippy,
smoke, security, and same-runner hosted benchmark checks passed.

No other runtime improvement is claimed by this release.

## Release Gates

```text
scripts/ai/verify_release.sh v33-maintenance-release
scripts/ai/verify_release.sh v33-strict-provenance
scripts/ai/verify_release.sh current-governance backend-freeze-baseline
scripts/ai/verify_release.sh rust-only-gates
scripts/ai/verify_release.sh release-publication-guard
scripts/ai/verify_release.sh release-publish-after-gate
```

The public GitHub Release is created only after the tag-push-triggered full
hosted release gate succeeds for the same tag ref and commit. A manual workflow
run for the same SHA is not publication authority. Publication evidence uses
`source_tree_plus_github_remote`; generated local publication output is not
source-controlled and is never sufficient as the sole proof.

## Boundary

All 27 registered release boundary flags are explicit `false`.

```text
backend_go_live_claim = false
actual_backend_production_go_live_allowed = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
automatic_recovery_allowed = false
dashboard_trading_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
default_production_execution_allowed = false
frontend_completion_claim = false
product_grade_live_trading_terminal_claim = false
```

The complete flag set is authoritative in
`docs/rust-cutover/release/v0_33_0_release_manifest.json`.

## Rollback

Before publication, reject the release candidate and do not publish. After
publication, revert the release commit on `main` and use a separately scoped
corrective release if an artifact correction is required. Never rewrite the
published v0.33.0 tag or the frozen v0.32.0 baseline.

## Next Track

The next capability family is `v0.34.0+` and remains separately scoped only.
It inherits none of the forbidden production or trading-control capabilities
from v0.32.0 or v0.33.0.
