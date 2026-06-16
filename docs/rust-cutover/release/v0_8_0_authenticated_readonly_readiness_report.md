# NTPRO v0.8.0 Authenticated Read-Only Readiness Report

Date: 2026-06-16
Executor: Codex
Milestone: `ntpro-rust-only-v0.8.0` candidate
Status: RELEASE CANDIDATE READY FOR OWNER CLOSURE

## Summary

`v0.8.0` is ready to enter owner release-closure decision for a scoped
authenticated Binance testnet read-only proof.

Plain Chinese summary: v0.8.0 可以对外说“已经具备 Binance testnet authenticated
read-only proof 的发布候选能力”：能在人工 opt-in 后用 testnet API key 做只读
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
| V080-008 | PASS CANDIDATE | `docs/rust-cutover/evidence/V080-008.md` | This PR creates readiness and release-note candidate docs. |
| V080-009 | WAITING OWNER APPROVAL | pending | Formal tag/GitHub Release closure is not complete and must not be implied before owner approval. |

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

Hosted PR evidence:

- V080-006 PR #344: Rust Cutover Smoke PASS.
- V080-007 PR #345: Rust Cutover Smoke PASS and security-audit PASS.

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

## Release Blockers

No V080 implementation blocker remains for release-closure decision.

The only remaining release action is V080-009 owner-approved closure:

- confirm current `main`;
- confirm this readiness report and release notes candidate;
- create the `ntpro-rust-only-v0.8.0` tag only after owner approval;
- publish GitHub Release only after owner approval;
- record closure evidence.

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

## Final Candidate Verdict

`v0.8.0` is a release-closure candidate for authenticated Binance testnet
read-only proof only.

Do not publish the tag or GitHub Release until V080-009 receives explicit owner
approval.
