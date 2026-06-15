# NTPRO v0.7.1 Release Gate Hardening Readiness Report

Date: 2026-06-15
Executor: Codex
Milestone: v0.7.1 release-gate and artifact-contract hardening

## Decision

Status: DRAFT until hosted release gate passes.

v0.7.1 is a hardening release for the already published v0.7.0 read-only
Binance testnet connectivity proof. It does not add order submission,
authenticated account access, production Binance connectivity, real funds, or
production trading.

## Plain Chinese Summary

v0.7.1 修的是发布门禁和产物合同，不是新能力版本。

大白话：v0.7.0 已经有 Binance testnet 公开 HTTP 只读连通性证明，但 v0.7
自己的验证脚本没有接进正式 release gate 和 PR smoke。v0.7.1 把这条门禁链路补齐，
并把 HTTP probe 产物从“HTTP 2xx 就算成功”收紧为“HTTP 2xx 且 Binance
`serverTime` response shape 校验通过才算 connectivity proof”。

## Included

- Wire v0.7 default offline gate into `scripts/ai/verify_release.sh`.
- Wire v0.7 manual-online preflight into `scripts/ai/verify_release.sh`.
- Add v0.7 stages to hosted `Rust Cutover Release Gate`.
- Add v0.7 path classification and smoke steps to PR smoke.
- Add canonical HTTP probe artifact:
  `testnet/http_connectivity_probe.json`.
- Preserve compatible summary probe artifact:
  `testnet/connectivity_probe.json`.
- Validate Binance `/api/v3/time` response shape before setting
  `testnet_connection=true`.
- Add explicit connection-boundary fields:
  `production_venue_connection`, `testnet_public_network_connection`, and
  `external_network_attempted`.
- Treat stable classified HTTP failures as diagnostic evidence, not as
  connectivity proof.
- Remove v06 identity strings from current v0.7 testnet workflow IDs.
- Update Roadmap to current public release `ntpro-rust-only-v0.7.0`, active
  hardening `v0.7.1`, and next capability `v0.8.0`.

## Not Included

- Testnet order submission.
- Order cancel, replace, amend, or live order management.
- Authenticated Binance testnet account proof.
- Account mutation.
- Production Binance connectivity.
- Production trading parity.
- Real funds.
- Dashboard network initiation.
- Dashboard credential access.
- Credential config field rename for the authenticated read-only v0.8 track.
- Prebuilt binary or Docker delivery.

## Required Validation

Local validation:

```text
cargo fmt --check
cargo test -p nautilus-cli workflow --lib
cargo test -p nautilus-cli dashboard --lib
scripts/ai/verify_v06_binance_testnet_dry_run.sh
scripts/ai/verify_v07_default_offline_gate.sh
scripts/ai/verify_v07_manual_online_gate.sh
scripts/ai/verify_release.sh v07-default-offline-gate v07-manual-online-preflight
scripts/ai/verify_fast.sh
git diff --check
```

Hosted validation:

```text
Rust Cutover Release Gate on the v0.7.1 release commit
```

## Release Boundary

```text
v0.7.1 = hardening only
current public capability = v0.7 read-only Binance testnet connectivity proof
next capability = v0.8 authenticated read-only testnet proof
no orders
no real funds
no production trading
```
