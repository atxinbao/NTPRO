# NTPRO Rust-only v0.7.1 Release Notes

Date: 2026-06-15
Executor: Codex
Release line: v0.7.1
Status: Released

## Release Identity

```text
Source tag: ntpro-rust-only-v0.7.1
Capability: v0.7 read-only Binance testnet connectivity proof hardening
Boundary: release-gate and artifact-contract hardening only
```

`ntpro-rust-only-v0.7.1` was published as a formal GitHub Release after the
hosted release gate passed on the exact release commit.

## Plain Chinese Summary

`v0.7.1` 是 `v0.7.0` 的修复版，不是下单能力版本。

这次修复的重点是：把 v0.7 的默认离线 gate 和 manual-online preflight 接进正式
release gate / PR smoke，并把 HTTP probe 成功条件收紧到 Binance
`serverTime` response shape 已验证。稳定失败现在只算诊断分类，不算 connectivity
proof。

## Included

- v0.7 release verifier stages:
  - `v07-default-offline-gate`;
  - `v07-manual-online-preflight`.
- Hosted release gate stages:
  - `release-v07-default-offline-gate`;
  - `release-v07-manual-online-preflight`.
- PR smoke v0.7 path detection and v0.7 smoke steps.
- Canonical HTTP artifact:
  - `testnet/http_connectivity_probe.json`;
  - `ntpro.v07_binance_testnet_http_probe.v1`.
- Compatible summary probe artifact remains:
  - `testnet/connectivity_probe.json`;
  - `ntpro.v07_binance_testnet_connectivity_probe.v1`.
- HTTP success requires validated Binance server-time response shape:
  - `response_shape=binance_server_time_v1`;
  - `response_shape_validated=true`.
- Workflow artifacts now separate:
  - `production_venue_connection`;
  - `testnet_public_network_connection`;
  - `external_network_attempted`.
- Current v0.7 testnet workflow identities no longer use v06 IDs.
- Roadmap aligned to:
  - current public release: `ntpro-rust-only-v0.7.1`;
  - next capability: `v0.8.0`.

## Hardening PRs

```text
V071-001 / #318: release gate and PR smoke wiring
V071-002 / #319: roadmap and release-facing docs alignment
V071-003 / #320: HTTP probe artifact contract normalization
V071-004 / #321: Binance serverTime response-shape validation
V071-005 / #322: manual-online proof semantics
V071-006 / #323: current v0.7 identity cleanup
V071-007 / #324: production/testnet/network field clarification
V071-008: readiness and release notes closeout
```

## Not Included

`v0.7.1` does not claim these capabilities:

- testnet order submission;
- order cancel, replace, amend, or live order management;
- authenticated Binance testnet account access;
- account mutation;
- production Binance connectivity;
- production trading;
- real funds;
- Dashboard network initiation;
- Dashboard credential access;
- credential config field rename for v0.8 authenticated read-only proof;
- prebuilt binary or Docker delivery.

## Migration Note Status

No trading migration note is required. `v0.7.1` changes release verification and
local workflow artifact contracts only. It does not change trading semantics or
enable any order path.

## Published Release Boundary

Published from exact release commit
`d20179301c99f05b4f11c6d4ee67ee48c7b7768a` after both hosted release gates
passed. The release remains hardening only: no order submission, no real funds,
no production Binance connectivity, and no production trading.

## Release Closure Evidence

```text
workflow_dispatch Release Gate:
  https://github.com/atxinbao/NTPRO/actions/runs/27549225688
  status: PASS
  headSha: d20179301c99f05b4f11c6d4ee67ee48c7b7768a

tag-triggered Release Gate:
  https://github.com/atxinbao/NTPRO/actions/runs/27552805993
  status: PASS
  headSha: d20179301c99f05b4f11c6d4ee67ee48c7b7768a

GitHub Release:
  https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.7.1
  isDraft: false
  isPrerelease: false
  publishedAt: 2026-06-15T15:17:01Z
```

Optional manual online proof is not a default CI or release blocker. Owners may
run it explicitly with:

```bash
NTPRO_V07_MANUAL_ONLINE=1 NTPRO_ALLOW_TESTNET_NETWORK=1 scripts/ai/verify_v07_manual_online_gate.sh
```
