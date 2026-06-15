# NTPRO v0.7.1 Release Gate Hardening Readiness Report

Date: 2026-06-15
Executor: Codex
Milestone: v0.7.1 release-gate and artifact-contract hardening

## Decision

Status: RELEASED.

v0.7.1 is a hardening release for the already published v0.7.0 read-only
Binance testnet connectivity proof. It does not add order submission,
authenticated account access, production Binance connectivity, real funds, or
production trading.

The owner-approved release closure created tag `ntpro-rust-only-v0.7.1` and
published the formal GitHub Release after hosted gates passed on the exact
release commit.

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
| V071-008 | merged | Release notes/readiness closeout. |
| V072-001 | this post-release wording PR | Replace pre-release candidate wording with released wording. |
| V072-002 / #327 | merged | README and ROADMAP current release aligned to v0.7.1. |
| V072-003 / #328 | merged | Online HTTP read-only boundary notes corrected for explicit opt-in socket attempts. |
| V072-004 | this release-closure evidence PR | Hosted gate, tag-triggered gate, release URL, and optional manual-online proof location recorded. |

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
- Roadmap is aligned to current public release `ntpro-rust-only-v0.7.1` and
  next capability `v0.8.0`.

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
Workflow-dispatch Rust Cutover Release Gate on the final v0.7.1 release commit:
PASS.
Tag-triggered Rust Cutover Release Gate on `ntpro-rust-only-v0.7.1`: PASS.
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

The exact release commit was verified before tag creation.

Post-release closure evidence:

```text
exact release commit = d20179301c99f05b4f11c6d4ee67ee48c7b7768a
workflow_dispatch Release Gate = https://github.com/atxinbao/NTPRO/actions/runs/27549225688
workflow_dispatch status = PASS
tag-triggered Release Gate = https://github.com/atxinbao/NTPRO/actions/runs/27552805993
tag-triggered status = PASS
formal tag = ntpro-rust-only-v0.7.1
formal GitHub Release = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.7.1
release status = published, not draft, not prerelease
```

Optional manual online proof remains owner-run evidence only:

```bash
NTPRO_V07_MANUAL_ONLINE=1 NTPRO_ALLOW_TESTNET_NETWORK=1 scripts/ai/verify_v07_manual_online_gate.sh
```

The default release gate and PR smoke must not depend on real Binance network
availability. Without both explicit opt-ins, the manual-online gate remains a
CI-safe preflight that stops before socket creation.

## Release Boundary

```text
v0.7.1 = hardening only
current public capability = v0.7 read-only Binance testnet connectivity proof
next capability = v0.8 authenticated read-only testnet proof
no orders
no real funds
no production trading
no Dashboard network initiation
formal tag = ntpro-rust-only-v0.7.1
formal GitHub Release = NTPRO Rust-only v0.7.1
```
