# BFR-001 Manual-Online Production Mutation Retirement

Date: 2026-07-29
Executor: Codex
Task: BFR-001 / GitHub issue #1166

## Superseded Status

PAR-007 supersedes the wrapper-only behavior described by BFR-001. The current
CLI no longer accepts `--manual-online`, `--api-key-env`, or
`--api-secret-env` for either legacy command, and the corresponding credential,
signing, POST, and DELETE executor paths have been removed from the product
binary.

## Change

BFR-001 originally made the `nautilus live` CLI reject these legacy
manual-online production order submit and actual-cancel invocations before
reading files, environment variables, or signing material:

- `production-mutation-guarded-send --manual-online`;
- `production-mutation-actual-cancel-single-shot --manual-online`.

As of PAR-007, both commands are offline-only historical artifact evaluators.
Their online and credential options are absent from the parser, and no
production mutation HTTP executor remains behind them. Production
public/account/order-state read-only probes are unchanged.

中文说明：v0.32.0 后端冻结没有授予后续版本生产下单、撤单、adapter send 或 live
exchange request 权限。BFR-001 最初在最外层拒绝上述两个 `--manual-online`
入口；PAR-007 进一步删除了相关在线参数、凭证读取、签名和真实 POST/DELETE
executor。原有离线历史工件检查仍可继续使用。

## Operator Action

Remove the retired online and credential flags from any retained audit or
replay command. Do not treat historical v0.16-v0.19 artifacts, release
manifests, or owner approvals as authorization for current production
execution.

Any future live trading capability requires a separately scoped release,
explicit owner/operator authorization, new current-version provenance, and
dedicated runtime, risk, rollback, telemetry, and negative-test evidence.

## Rollback

No automatic rollback is allowed. Restoring network mutation requires a new
capability decision and must not inherit authority from the frozen backend.
