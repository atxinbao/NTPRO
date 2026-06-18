# NTPRO v0.9.1 Strategy Runtime Hardening Readiness Report

Date: 2026-06-18
Executor: Codex
Milestone: `ntpro-rust-only-v0.9.1`
Status: PLANNING - NOT READY FOR RELEASE

## Summary

`v0.9.1` is the Strategy Runtime Semantics & Audit Hardening patch for the
published v0.9 local deterministic Strategy Runtime batch foundation line.

Plain Chinese summary: v0.9.1 不是 Binance 下单版本，也不是实盘版本。它是在 v0.9.0
本地批处理策略证明之后，先把 node/session/market/risk/heartbeat/artifact 语义补真实，
避免在一个“策略已经 stopped 但 node 还 running”的基础上直接接订单。v0.10.0 才是
最早的 Binance testnet order proof 版本。

## Product Claim

```text
capability = Strategy Runtime Semantics & Audit Hardening
current published release = ntpro-rust-only-v0.9.0
next patch track = v0.9.1
next capability track = v0.10.0 Binance Testnet Order Proof
runtime code changes = scoped to strategy runtime semantics only
order submission = not included
production trading = not included
```

## Patch Scope

Included:

```text
release-surface correction
StrategyNode config contract unification
persistent StrategySession lifecycle semantics
node/session/market status coherence
runtime counter snapshot semantics
kill-switch enabled/active split
Strategy Session manifest and child artifact audit
Supervisor/Dashboard degraded status visibility
integration, heartbeat, shutdown, and restart smoke coverage
final v0.9.1 readiness and release-note closure
```

Not included:

```text
adapter behavior
trading semantics
release tag
GitHub Release publication
Binance testnet order proof
```

## Task Gate Matrix

| Task | Status | Evidence | Notes |
| --- | --- | --- | --- |
| V091-001 | PASS | `docs/rust-cutover/evidence/V091-001.md` | Public release surface now points at v0.9.0 released state and frames v0.9.1 as a correction patch. |
| V091-002 | PENDING | TBD | Unify StrategyNode config loading and validation. |
| V091-003 | PENDING | TBD | Make StrategySession lifecycle persistent until stop/pause/risk halt. |
| V091-004 | PENDING | TBD | Correct node/session/market lifecycle mapping. |
| V091-005 | PENDING | TBD | Add shared runtime counters and prevent heartbeat regression. |
| V091-006 | PENDING | TBD | Split kill-switch enabled and active semantics. |
| V091-007 | PENDING | TBD | Add Strategy Session manifest and artifact audit. |
| V091-008 | PENDING | TBD | Surface artifact errors and state conflicts in Supervisor/Dashboard. |
| V091-009 | PENDING | TBD | Add integration, heartbeat, shutdown, and restart smoke coverage. |
| V091-010 | PENDING | TBD | Final v0.9.1 readiness, release notes, and closure. |

## Validation Evidence

Local validation collected so far:

```text
scripts/ai/verify_fast.sh
rg release-surface checks
git diff --check
```

Final v0.9.1 release validation is not complete. It must include the dedicated
V091 runtime, artifact, Supervisor/Dashboard, shutdown, and restart smoke gates
before owner release decision.

## Release Closure Status

This report does not create a tag and does not publish a GitHub Release. It is
the planning/readiness tracker for a possible owner-approved
`ntpro-rust-only-v0.9.1` release after all V091 tasks complete.

If the owner approves publication later, the release must preserve this
boundary:

```text
Strategy Runtime semantics and audit hardening only
current published capability remains v0.9 local deterministic batch foundation
v0.10.0 is earliest Binance testnet order proof track
no Binance testnet order submission
no real funds
no production trading
no Dashboard order controls
```

## Final Verdict

`v0.9.1` is not ready for owner release decision yet. V091-002 through V091-010
must complete first.

Do not describe this readiness PASS as Binance testnet order readiness,
real-funds readiness, production trading readiness, or Dashboard order-control
readiness.
