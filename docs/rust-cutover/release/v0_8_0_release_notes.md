# NTPRO Rust-only v0.8.0 Release Notes

Date: 2026-06-16
Executor: Codex
Status: PUBLISHED

## Summary

`v0.8.0` is scoped to authenticated Binance testnet read-only proof.

Plain Chinese summary: 这个版本只证明一件事：在人工明确开启 testnet 网络和提供
testnet API key/secret 后，NTPRO 可以做一个只读的 Binance testnet authenticated
`GET /api/v3/account` 响应 shape 证明，并且只留下脱敏 artifact。它不是实盘交易版本，
不支持真实资金，不支持生产 Binance，不支持下单、撤单、改单，也不允许 Dashboard
主动发起网络 probe。

## Added

- Authenticated Binance testnet read-only proof artifact:
  - `testnet/authenticated_readonly_probe.json`;
  - schema `ntpro.v08_binance_testnet_authenticated_readonly_probe.v1`;
  - redacted status, endpoint, method, response-shape, and boundary fields.
- Signed request builder guard for the only allowed authenticated endpoint:
  - `GET /api/v3/account` on Binance testnet.
- Env-only credential policy for testnet keys:
  - `BINANCE_TESTNET_API_KEY`;
  - `BINANCE_TESTNET_API_SECRET`.
- Synthetic secret leak scanner for v0.8 generated output.
- v0.8 default offline gate.
- v0.8 authenticated read-only preflight/manual online gate.
- Dashboard read-only display for authenticated proof artifacts.
- Release/PR smoke wiring for v0.8 default offline and authenticated preflight gates.

## Boundary

Included:

```text
authenticated Binance testnet read-only proof
manual-online-only authenticated account-shape validation
env-only credentials
redacted artifact evidence
synthetic secret leak checks
Dashboard read-only artifact display
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

Release validation includes:

```bash
scripts/ai/verify_v08_default_offline_gate.sh
scripts/ai/verify_v08_authenticated_readonly_gate.sh
scripts/ai/verify_release.sh v08-default-offline-gate v08-authenticated-readonly-preflight
scripts/ai/scan_v08_synthetic_secret_leaks.sh
```

Default CI/release validation remains offline and does not require real Binance
credentials.

## Manual Online Proof

Manual online proof requires explicit owner action:

```bash
NTPRO_V08_MANUAL_ONLINE=1 \
NTPRO_ALLOW_TESTNET_NETWORK=1 \
BINANCE_TESTNET_API_KEY=<testnet key> \
BINANCE_TESTNET_API_SECRET=<testnet secret> \
scripts/ai/verify_v08_authenticated_readonly_gate.sh
```

Manual online success means authenticated Binance testnet account response shape
was validated. It does not mean production trading readiness.

## Release Status

Tag: `ntpro-rust-only-v0.8.0`
Release name: `NTPRO Rust-only v0.8.0`
Release URL:
`https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.8.0`
Published at: `2026-06-16T20:20:57Z`
Release flags: `isDraft=false`, `isPrerelease=false`

This is a formal GitHub Release. The release boundary remains authenticated
Binance testnet read-only proof only: no production Binance, no real funds, no
production trading, and no order submission.
