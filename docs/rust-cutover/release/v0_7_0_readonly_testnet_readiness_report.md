# NTPRO v0.7.0 Read-Only Binance Testnet Readiness Report

Date: 2026-06-15
Executor: Codex
Task: V070-007
Decision: PASS for scoped v0.7.0 release closure preparation

## Plain Chinese Summary

v0.7.0 的范围已经可以收口为：

```text
real Binance testnet read-only connectivity proof
```

大白话说：这一版证明 NTPRO 可以在明确人工 opt-in 后，访问 Binance testnet 的公开只读
HTTP endpoint，并把结果写成可审计 artifact。默认本地和 CI 仍然离线。Dashboard 只读取已经
存在的 artifact，不会自己发起联网探测，也不会读取密钥。

这不是实盘能力，也不是 testnet 下单能力。v0.7.0 不提交订单、不撤单、不改单、不使用真实资金、
不声明生产 Binance connectivity，也不声明生产交易 parity。

## Release Scope

Included capability:

- Real Binance testnet public HTTP read-only connectivity proof.
- Fail-closed network opt-in guard:
  - `--allow-testnet-network`;
  - `NTPRO_ALLOW_TESTNET_NETWORK=1`;
  - testnet config;
  - disabled order submission;
  - `real_orders_submitted=false`.
- Environment-only credential policy:
  - credential values are never stored;
  - only env var names, presence booleans, policy labels, and redaction status
    may be recorded.
- HTTP probe artifact with success or stable classified failure.
- Optional/manual WebSocket probe artifact schema.
- Dashboard read-only display of generated V07 probe artifacts.
- Dual verification:
  - default offline gate;
  - manual online gate.

Excluded capability:

- Testnet order submission.
- Testnet account mutation.
- Production Binance connectivity.
- Production trading.
- Real funds.
- Production parity.
- Dashboard connect/order/cancel/amend controls.
- Dashboard credential access.
- WebSocket subscription engine.
- Git tag or GitHub Release publication in this task.

## Evidence Matrix

| Task | Scope | Evidence | PR status |
| --- | --- | --- | --- |
| V070-000 | Read-only testnet boundary and threat model | `docs/rust-cutover/evidence/V070-000.md` | Completed before this report |
| V070-001 | Fail-closed network opt-in guard | `docs/rust-cutover/evidence/V070-001.md` | Completed before this report |
| V070-002 | Environment-only credential policy | `docs/rust-cutover/evidence/V070-002.md` | Completed before this report |
| V070-003 | HTTP read-only testnet probe artifact | `docs/rust-cutover/evidence/V070-003.md` | Completed before this report |
| V070-004 | Optional/manual WebSocket probe artifact | `docs/rust-cutover/evidence/V070-004.md` | Completed before this report |
| V070-005 | Dashboard read-only probe artifact display | `docs/rust-cutover/evidence/V070-005.md` | Completed before this report |
| V070-006 | Offline/manual-online dual gate scripts | `docs/rust-cutover/evidence/V070-006.md` | Completed before this report |
| V070-007 | Readiness report and release notes draft | `docs/rust-cutover/evidence/V070-007.md` | Prepared by this PR |

## Gate Results

Default offline gate:

```text
script: scripts/ai/verify_v07_default_offline_gate.sh
status: PASS
dry_run_network_attempted=false
blocked_probe_network_attempted=false
dry_run_status=dry_run_completed
blocked_probe_status=offline_probe_validated
```

Manual online fail-closed preflight:

```text
script: scripts/ai/verify_v07_manual_online_gate.sh
status: PASS
network_permission_requested=true
network_attempted=false
reason=NTPRO_ALLOW_TESTNET_NETWORK=1 is not set
```

Manual online read-only HTTP proof:

```text
command: NTPRO_V07_MANUAL_ONLINE=1 NTPRO_ALLOW_TESTNET_NETWORK=1 scripts/ai/verify_v07_manual_online_gate.sh
status: PASS
network_attempted=true
testnet_connection=true
error_code=none
real_orders_submitted=false
values_recorded=false
secrets_redacted=true
```

Fast local gate:

```text
script: scripts/ai/verify_fast.sh
status: PASS
```

## Boundary Checks

PASS criteria:

- Default CI/local gate does not open sockets.
- Real online proof requires explicit manual opt-in.
- Real online proof is HTTP read-only only.
- No order path is enabled.
- No real order is submitted.
- No real funds are used.
- Credential values are not stored or printed.
- Dashboard remains artifact-only.
- Release wording says what is included and not included.

Current result:

```text
PASS
```

## Remaining Risks For Next Versions

- WebSocket remains optional/manual artifact schema only. It is not yet a
  subscription engine and is not a release blocker.
- No testnet order submission is approved. If a future version adds testnet
  order dry-run or testnet order proof, it needs a separate scope decision and
  stronger adapter evidence.
- Production Binance connectivity and production trading parity remain out of
  scope.
- Dashboard remains a viewer. Turning it into an actuator requires separate
  control-surface design, auth model, and failure-mode evidence.

## Final Readiness Decision

v0.7.0 is ready for owner release decision as a scoped read-only Binance testnet
connectivity proof.

This report does not create a tag and does not publish a GitHub Release.
