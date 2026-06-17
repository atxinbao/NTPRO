# NTPRO Rust-only v0.8.1 Release Notes

Date: 2026-06-17
Executor: Codex
Status: READY FOR OWNER RELEASE DECISION

## Summary

`v0.8.1` is a safety/closure patch for the published v0.8 authenticated Binance
testnet read-only proof. It does not add order placement, account mutation, or
production Binance support.

Plain Chinese summary: v0.8.1 只是在 v0.8.0 基础上补安全边界和证据口径，不是实盘版本。
它继续只允许“人工明确开启后的 Binance testnet authenticated read-only proof”。它不支持真实资金，
不支持生产 Binance，不支持下单、撤单、改单，不允许 Dashboard 主动发起网络 probe。

## Changed

- Release/public wording now points at the published v0.8.0 state and frames
  v0.8.1 as a safety/closure patch.
- Authenticated runtime probe execution now requires explicit
  `NTPRO_V08_MANUAL_ONLINE=1` in addition to the existing testnet network
  opt-in and credential requirements.
- `summary.json` and `manifest.json` now expose authenticated proof status:
  - `authenticated_probe_attempted`;
  - `authenticated_readonly_probe_status`;
  - `authenticated_response_shape_validated`;
  - `authenticated_connectivity_proof`.
- Authenticated account response-shape metadata now uses
  `binance_account_readonly_redacted_v1`.
- Account response-shape success now requires `canWithdraw` and `canDeposit`
  booleans in addition to the previous bounded shape fields.
- The v0.8 authenticated read-only gate now expects the hardened redacted
  response-shape label.

## Boundary

Included:

```text
authenticated Binance testnet read-only proof safety hardening
manual-online-only runtime gate
redacted account response-shape metadata
summary/manifest proof status visibility
release/readiness documentation
```

Not included:

```text
testnet order placement
order cancel/replace/amend
account mutation
production Binance connectivity
production trading
real funds
production parity
Dashboard-started probes
Dashboard credential entry
raw account body persistence
balance/uid/account-identifying persistence
```

## Validation

Readiness evidence for the V081 patch set includes:

```bash
scripts/ai/verify_fast.sh
scripts/ai/verify_v08_authenticated_readonly_gate.sh
```

Hosted smoke evidence:

```text
PR #348 Rust Cutover Smoke PASS
PR #349 Rust Cutover Smoke PASS
PR #350 Rust Cutover Smoke PASS, run 27694046977
PR #351 Rust Cutover Smoke PASS, run 27697777842
```

Default validation remains offline and does not require real Binance
credentials.

## Manual Online Proof

Manual online proof remains owner opt-in only:

```bash
NTPRO_V08_MANUAL_ONLINE=1 \
NTPRO_ALLOW_TESTNET_NETWORK=1 \
BINANCE_TESTNET_API_KEY=<testnet key> \
BINANCE_TESTNET_API_SECRET=<testnet secret> \
scripts/ai/verify_v08_authenticated_readonly_gate.sh
```

Manual online success means authenticated Binance testnet account response
shape was validated and redacted artifact status was recorded. It does not mean
production trading readiness.

## Release Status

This document prepares release notes for a possible
`ntpro-rust-only-v0.8.1` release decision.

This task does not create a tag and does not publish a GitHub Release.

If owner-approved publication happens later, the release name should remain:

```text
NTPRO Rust-only v0.8.1
```

and the release boundary must remain authenticated Binance testnet read-only
proof safety/closure only: no production Binance, no real funds, no production
trading, no order submission, and no account mutation.
