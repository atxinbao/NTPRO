# BFR-001 Manual-Online Production Mutation Retirement

Date: 2026-07-29
Executor: Codex
Task: BFR-001 / GitHub issue #1166

## Change

The current `nautilus live` CLI no longer executes the legacy manual-online
production order submit or actual-cancel paths:

- `production-mutation-guarded-send --manual-online`;
- `production-mutation-actual-cancel-single-shot --manual-online`.

Both commands remain available without `--manual-online` for deterministic,
redacted, offline artifact evaluation. Production public/account/order-state
read-only probes are unchanged.

中文说明：v0.32.0 后端冻结没有授予后续版本生产下单、撤单、adapter send 或 live
exchange request 权限。因此当前 CLI 对上述两个 `--manual-online` 入口直接报错，
且在读取输入文件、环境凭证或签名材料之前停止。原有离线工件检查仍可继续使用。

## Operator Action

Remove `--manual-online` from any retained audit or replay command. Do not treat
historical v0.16-v0.19 artifacts, release manifests, or owner approvals as
authorization for current production execution.

Any future live trading capability requires a separately scoped release,
explicit owner/operator authorization, new current-version provenance, and
dedicated runtime, risk, rollback, telemetry, and negative-test evidence.

## Rollback

No automatic rollback is allowed. Restoring network mutation requires a new
capability decision and must not inherit authority from the frozen backend.
