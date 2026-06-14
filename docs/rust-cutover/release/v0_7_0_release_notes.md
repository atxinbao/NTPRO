# NTPRO Rust-only v0.7.0 Release Notes

Date: 2026-06-15
Executor: Codex
Release line: v0.7.0
Status: Final release notes

## Release Identity

```text
Current source tag: ntpro-rust-only-v0.7.0
Capability: real Binance testnet read-only connectivity proof
Boundary: no orders, no real funds, no production trading
```

## Plain Chinese Summary

`v0.7.0` 是 NTPRO Rust-only 在 Binance testnet 方向上的第一轮真实只读连通性证明。

这一版可以在明确人工打开 online gate 后，访问 Binance testnet 的公开只读 HTTP endpoint，
并把成功或稳定失败原因写成 artifact。默认本地和 CI 仍然离线，不会偷偷联网。

这不是 testnet 下单版本，更不是实盘版本。它不会提交真实订单，不使用真实资金，不声明
production Binance connectivity，也不声明 production trading parity。

## Included

- Real Binance testnet public HTTP read-only connectivity proof.
- Explicit network opt-in:
  - `--allow-testnet-network`;
  - `NTPRO_ALLOW_TESTNET_NETWORK=1`;
  - manual online gate command.
- Fail-closed behavior when opt-in is missing.
- Environment-only credential policy:
  - no credential values in config files;
  - no credential values in artifacts;
  - no credential values in stdout/logs.
- HTTP probe artifact with:
  - `network_attempted=true` when real probe runs;
  - `testnet_connection=true` on successful public testnet response;
  - stable `error_code` on classified failure.
- Optional/manual WebSocket probe artifact schema.
- Dashboard read-only display of generated HTTP/WebSocket probe artifacts.
- Dual verification scripts:
  - `scripts/ai/verify_v07_default_offline_gate.sh`;
  - `scripts/ai/verify_v07_manual_online_gate.sh`.

## Not Included

`v0.7.0` does not claim these capabilities:

- testnet order submission;
- order cancel, replace, amend, or live order management;
- testnet account mutation;
- production Binance connectivity;
- production trading;
- real funds;
- production parity;
- Dashboard network initiation;
- Dashboard credential access;
- Dashboard connect/order/cancel/amend controls;
- WebSocket subscription engine;
- prebuilt binary or Docker delivery as a release requirement.

## Validation Source

Primary readiness evidence:

- `docs/rust-cutover/release/v0_7_0_readonly_testnet_readiness_report.md`
- `docs/rust-cutover/release/v0_7_0_readonly_testnet_boundary.md`
- `docs/rust-cutover/evidence/V070-000.md` through
  `docs/rust-cutover/evidence/V070-007.md`

Verification:

- `scripts/ai/verify_v07_default_offline_gate.sh`: PASS
- `scripts/ai/verify_v07_manual_online_gate.sh`: PASS, fail-closed preflight
- `NTPRO_V07_MANUAL_ONLINE=1 NTPRO_ALLOW_TESTNET_NETWORK=1 scripts/ai/verify_v07_manual_online_gate.sh`: PASS
- `scripts/ai/verify_fast.sh`: PASS
- release wording scan: PASS
- `git diff --check`: PASS

## Manual Online Command

Real read-only HTTP proof requires explicit owner/operator opt-in:

```bash
NTPRO_V07_MANUAL_ONLINE=1 \
NTPRO_ALLOW_TESTNET_NETWORK=1 \
scripts/ai/verify_v07_manual_online_gate.sh
```

Expected boundary:

```text
network_attempted=true
real_orders_submitted=false
values_recorded=false
secrets_redacted=true
```

## Boundary

```text
v0.7.0 read-only Binance testnet connectivity proof
no testnet order submission
no production Binance connectivity
no real funds
no production trading
no production parity
```

## Migration Note Status

No migration note is required. `v0.7.0` adds verification/reporting artifacts and
Dashboard read-only display of those artifacts. It does not change trading
semantics or public order APIs.

## Release Action Status

These notes are intended for the `ntpro-rust-only-v0.7.0` GitHub Release.
