# NTPRO v0.7.1 Release Gate Hardening Readiness Report

Date: 2026-06-15
Executor: Codex
Milestone: v0.7.1 release-gate and artifact-contract hardening

## Decision

Status: READY FOR RELEASE CLOSURE CANDIDATE.

v0.7.1 is a hardening release for the already published v0.7.0 read-only
Binance testnet connectivity proof. It does not add order submission,
authenticated account access, production Binance connectivity, real funds, or
production trading.

This report does not create the `ntpro-rust-only-v0.7.1` tag and does not
publish a GitHub Release. Release closure still requires an explicit owner
decision after the final PR is merged.

## Plain Chinese Summary

v0.7.1 修的是发布门禁和产物合同，不是新能力版本。

大白话：v0.7.0 已经有 Binance testnet 公开 HTTP 只读连通性证明，但 v0.7
自己的验证脚本没有接进正式 release gate 和 PR smoke。v0.7.1 把这条门禁链路补齐，
并把 HTTP probe 产物从“HTTP 2xx 就算成功”收紧为“HTTP 2xx 且 Binance
`serverTime` response shape 校验通过才算 connectivity proof”。

## Included

| Task / PR | Status | Scope |
| --- | --- | --- |
| V071-001 / #318 | merged | Release gate and PR smoke wiring for v0.7 stages. |
| V071-002 / #319 | merged | Roadmap and release-facing docs aligned to v0.7.1 hardening. |
| V071-003 / #320 | merged | Canonical HTTP probe artifact path shared by writer and Dashboard reader. |
| V071-004 / #321 | merged | Binance `/api/v3/time` success requires `serverTime` response-shape validation. |
| V071-005 / #322 | merged | Stable HTTP failures are diagnostic classifications, not connectivity proof. |
| V071-006 / #323 | merged | Current v0.7 workflow identities use read-only testnet names, not v06 runtime-foundation names. |
| V071-007 / #324 | merged | Dashboard and artifacts distinguish production connectivity, testnet read-only proof, and external network attempts. |
| V071-008 | this readiness PR | Release notes/readiness closeout only; no tag or GitHub Release. |

The included hardening scope is:

- `scripts/ai/verify_release.sh` can run v0.7 default offline and manual-online
  preflight stages.
- Hosted `Rust Cutover Release Gate` includes the v0.7 release stages.
- PR smoke classifies v0.7 path changes and runs the relevant v0.7 smoke steps.
- The canonical HTTP probe artifact is
  `testnet/http_connectivity_probe.json` with schema
  `ntpro.v07_binance_testnet_http_probe.v1`.
- The compatibility probe artifact remains
  `testnet/connectivity_probe.json`.
- HTTP success requires validated Binance server-time response shape before
  setting `testnet_connection=true`.
- Workflow artifacts and Dashboard use explicit connection-boundary fields:
  `production_venue_connection`, `testnet_public_network_connection`, and
  `external_network_attempted`.
- Stable classified HTTP failures remain diagnostic evidence only.
- Current v0.7 testnet workflow identities no longer use v06 runtime-foundation
  names.
- Roadmap is aligned to current public release `ntpro-rust-only-v0.7.0`, active
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
PR smoke for each hardening PR: PASS.
Rust Cutover Release Gate on the final v0.7.1 release commit: required during
explicit release closure.
```

## Validation Evidence

Local evidence expected for this readiness PR:

```text
scripts/ai/verify_release.sh v07-default-offline-gate v07-manual-online-preflight
scripts/ai/verify_fast.sh
git diff --check
```

Hardening PR smoke evidence:

```text
#319 PASS
#320 PASS
#321 PASS
#322 PASS
#323 PASS
#324 PASS
```

Final release closure must verify the exact release commit before tag creation.

## Release Boundary

```text
v0.7.1 = hardening only
current public capability = v0.7 read-only Binance testnet connectivity proof
next capability = v0.8 authenticated read-only testnet proof
no orders
no real funds
no production trading
no Dashboard network initiation
no tag creation in this readiness PR
no GitHub Release publication in this readiness PR
```
