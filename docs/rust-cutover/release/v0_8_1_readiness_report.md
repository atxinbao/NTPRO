# NTPRO v0.8.1 Safety/Closure Readiness Report

Date: 2026-06-17
Executor: Codex
Milestone: `ntpro-rust-only-v0.8.1`
Status: READY FOR OWNER RELEASE DECISION

## Summary

`v0.8.1` is a safety/closure patch for the published v0.8 authenticated Binance
testnet read-only proof. It does not add a new trading capability.

Plain Chinese summary: v0.8.1 不是新功能版本，只是把 v0.8.0 发布后发现的几个
边界点补严：文案明确 v0.8.0 已发布，authenticated probe 必须手动 online 授权，summary
和 manifest 能直接看到 authenticated proof 状态，account response shape 的名字和校验更清楚。
它仍然不是实盘版本，不支持真实资金，不支持生产 Binance，不支持下单、撤单、改单，也不允许
Dashboard 主动发起网络 probe。

## Product Claim

```text
capability = authenticated Binance testnet read-only proof safety closure
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

## Patch Scope

Included:

```text
release surface wording cleanup
manual-online runtime gate hardening
summary/manifest authenticated proof status promotion
authenticated account response-shape label hardening
v0.8 authenticated gate expectation sync
v0.8.1 release/readiness documentation
```

Not included:

```text
new Binance endpoint support
testnet order placement
order cancel/replace/amend
account mutation
production Binance connectivity
production trading
real funds
Dashboard-started probes
Dashboard credential entry
GitHub tag creation
GitHub Release publication
```

## Task Gate Matrix

| Task | Status | Evidence | Merge / Check Evidence |
| --- | --- | --- | --- |
| V081-001 | PASS | `docs/rust-cutover/evidence/V081-001.md` | PR #348 merged; public release wording updated after v0.8.0 publication. |
| V081-002 | PASS | `docs/rust-cutover/evidence/V081-002.md` | PR #349 merged; `NTPRO_V08_MANUAL_ONLINE=1` is required before authenticated runtime probe execution. |
| V081-003 | PASS | `docs/rust-cutover/evidence/V081-003.md` | PR #350 merged; GitHub smoke PASS run `27694046977`. |
| V081-004 | PASS | `docs/rust-cutover/evidence/V081-004.md` | PR #351 merged; GitHub smoke PASS run `27697777842`. |
| V081-005 | PASS | `docs/rust-cutover/evidence/V081-005.md` | This documentation closure task prepares release decision material only. |

## Validation Evidence

Local validation expected for the V081 closure set:

```text
scripts/ai/verify_fast.sh
scripts/ai/verify_v08_authenticated_readonly_gate.sh
rg checks for v0.8.1 boundary wording
git diff --check
```

Hosted validation already collected for code-bearing V081 tasks:

```text
PR #348 Rust Cutover Smoke PASS
PR #349 Rust Cutover Smoke PASS
PR #350 Rust Cutover Smoke PASS, run 27694046977
PR #351 Rust Cutover Smoke PASS, run 27697777842
```

## Manual Online Boundary

Manual online proof remains explicit only:

```bash
NTPRO_V08_MANUAL_ONLINE=1 \
NTPRO_ALLOW_TESTNET_NETWORK=1 \
BINANCE_TESTNET_API_KEY=<testnet key> \
BINANCE_TESTNET_API_SECRET=<testnet secret> \
scripts/ai/verify_v08_authenticated_readonly_gate.sh
```

Without `NTPRO_V08_MANUAL_ONLINE=1`, the authenticated runtime probe must not
execute, even if CLI flags, config, and the broader testnet network environment
are present.

## Response Shape Boundary

New V081/V0.8.1 authenticated artifacts use:

```text
response_shape = binance_account_readonly_redacted_v1
```

That label means the probe validates only bounded account response shape:
`accountType` is a string, `balances` is an array, and `canTrade`,
`canWithdraw`, and `canDeposit` are booleans. It does not permit storing raw
account body content, balances, account identifiers, permissions, commission
details, API keys, secrets, signatures, signed query strings, signed URLs, or
headers.

## Release Closure Status

This report does not create a tag and does not publish a GitHub Release. It is
the readiness packet for a possible owner-approved `ntpro-rust-only-v0.8.1`
release.

If the owner approves publication later, the release must preserve this
boundary:

```text
Binance testnet authenticated read-only proof only
no production Binance
no real funds
no production trading
no order submission
no account mutation
no Dashboard-started probes
```

## Final Verdict

`v0.8.1` is ready for owner release decision as a safety/closure patch only.

Do not describe this readiness PASS as production Binance readiness, real-funds
readiness, production trading readiness, order submission readiness, or a new
authenticated trading capability.
