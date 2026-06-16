# NTPRO v0.8.0 Authenticated Read-Only Readiness Report

Date: 2026-06-16
Executor: Codex
Milestone: `ntpro-rust-only-v0.8.0`
Status: RELEASED

## Summary

`v0.8.0` has completed owner-approved release closure for a scoped
authenticated Binance testnet read-only proof.

Plain Chinese summary: v0.8.0 已经正式发布，可以对外说“具备 Binance testnet
authenticated read-only proof 能力”：能在人工 opt-in 后用 testnet API key 做只读
`GET /api/v3/account` 账号响应 shape 证明，并把证明写成脱敏 artifact。它不能被说成
真实 Binance 实盘、生产交易、真实资金、下单、撤单、改单或生产账号能力。

## Product Claim

```text
capability = authenticated Binance testnet read-only proof
default CI/release mode = offline / credential-free / no network
manual online mode = explicit owner opt-in only
allowed authenticated endpoint = GET /api/v3/account on Binance testnet
artifact = redacted status and response-shape evidence only
production Binance connectivity = not included
order submission/cancel/replace/amend = not included
account mutation = not included
real funds = not included
production trading = not included
Dashboard network initiation = not included
```

## Task Gate Matrix

| Task | Status | Evidence | Notes |
| --- | --- | --- | --- |
| V080-000 | PASS | `docs/rust-cutover/evidence/V080-000.md` | Boundary and endpoint allowlist defined. |
| V080-001 | PASS | `docs/rust-cutover/evidence/V080-001.md` | Env-only credential policy and redaction fields defined. |
| V080-002 | PASS | `docs/rust-cutover/evidence/V080-002.md` | Authenticated artifact contract seeded without network behavior. |
| V080-003 | PASS | `docs/rust-cutover/evidence/V080-003.md` | Signed `GET /api/v3/account` request builder guarded by allowlist. |
| V080-004 | PASS | `docs/rust-cutover/evidence/V080-004.md` | Authenticated read-only artifact writer and response-shape validation added. |
| V080-005 | PASS | `docs/rust-cutover/evidence/V080-005.md` | Synthetic secret leak scanner and default offline gate added. |
| V080-006 | PASS | `docs/rust-cutover/evidence/V080-006.md` | Dashboard reads completed authenticated artifact only. |
| V080-007 | PASS | `docs/rust-cutover/evidence/V080-007.md` | Release/PR smoke wiring includes v08 default offline and authenticated preflight. |
| V080-008 | PASS | `docs/rust-cutover/evidence/V080-008.md` | Readiness and release-note candidate docs prepared. |
| V080-009 | PASS | `docs/rust-cutover/evidence/V080-009.md` | Owner-approved tag and formal GitHub Release closure completed. |

## Validation Evidence

The release candidate depends on these gates:

```text
scripts/ai/verify_v08_default_offline_gate.sh
scripts/ai/verify_v08_authenticated_readonly_gate.sh
scripts/ai/verify_release.sh v08-default-offline-gate v08-authenticated-readonly-preflight
scripts/ai/scan_v08_synthetic_secret_leaks.sh
scripts/ai/verify_fast.sh
```

Known local evidence from V080-007:

```text
v08_default_offline_gate status=ok
v08_authenticated_readonly_gate status=preflight_blocked
synthetic secret leak scan status=ok
verify_release v08-default-offline-gate v08-authenticated-readonly-preflight = PASS
```

Hosted PR/release evidence:

- V080-006 PR #344: Rust Cutover Smoke PASS.
- V080-007 PR #345: Rust Cutover Smoke PASS and security-audit PASS.
- Workflow-dispatch release gate:
  `https://github.com/atxinbao/NTPRO/actions/runs/27637885139` PASS.
- Tag-triggered release gate:
  `https://github.com/atxinbao/NTPRO/actions/runs/27641087223` PASS after
  rerunning failed jobs.
- GitHub Release:
  `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.8.0`.

## Manual Online Boundary

Manual online proof is not part of default CI. To run it, an owner must
explicitly provide all of:

```bash
NTPRO_V08_MANUAL_ONLINE=1
NTPRO_ALLOW_TESTNET_NETWORK=1
BINANCE_TESTNET_API_KEY=<testnet key>
BINANCE_TESTNET_API_SECRET=<testnet secret>
```

The manual online script still rejects production scope and records only
redacted artifact evidence. A successful manual online proof means the Binance
testnet authenticated account response shape was validated. It does not mean
production trading readiness.

## Release Closure

No V080 implementation blocker remains.

V080-009 owner-approved closure is complete:

- release commit: `f0da02717a498ce237459ffa6053a2f95800d4bc`;
- tag: `ntpro-rust-only-v0.8.0`;
- release name: `NTPRO Rust-only v0.8.0`;
- GitHub Release URL:
  `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.8.0`;
- release flags: `isDraft=false`, `isPrerelease=false`;
- published at: `2026-06-16T20:20:57Z`.

## Out Of Scope For v0.8.0

```text
testnet order placement
order cancel/replace/amend
listenKey/user-data-stream mutation
production Binance connectivity
production trading
real funds
production parity
Dashboard-started probes
Dashboard credential entry
prebuilt binary or Docker delivery
```

## Final Verdict

`v0.8.0` is formally released for authenticated Binance testnet read-only proof
only.

Do not describe this PASS as production Binance readiness, real-funds
readiness, production trading readiness, or order submission readiness.
