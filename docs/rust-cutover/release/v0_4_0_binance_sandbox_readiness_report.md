# NTPRO v0.4.0 Binance Sandbox Readiness Report

Date: 2026-06-13
Executor: Codex
Milestone: v0.4.0 Binance Sandbox Product Foundation
Decision: PASS

## Plain Chinese Summary

v0.4.0 的目标不是生产实盘，也不是“Binance 已经可以真实交易”。这一版只证明
一条本地、确定性、可审计的 Binance sandbox 产品基础路径：

```text
checked-in Binance fixture
  -> local replay
  -> mock order lifecycle
  -> deterministic risk rejection
  -> EMA / RSI strategy smokes
  -> local Dashboard exchange / strategy / order / risk panels
```

大白话说：这版可以用于本地演示和发布证据，证明 NTPRO 已经有一条 Binance
sandbox 产品骨架；但它不连接真实 Binance 账户，不使用真实资金，不提交真实
订单，也不声明生产 Binance parity。

Final decision: PASS for the scoped Binance sandbox product foundation. This
is still FAIL for production trading readiness by scope.

## Scope Source

- `docs/rust-cutover/scope/v0_4_0_binance_sandbox_product_foundation.md`

Release claim:

```text
Binance Sandbox Product Foundation
```

In scope:

- Binance Spot sandbox product boundary;
- Binance capability matrix with USDT-M deferred unless explicitly proven;
- checked-in Binance fixture replay;
- mock order lifecycle;
- deterministic halted-state risk rejection;
- built-in EMA and RSI sandbox strategy contracts and smokes;
- local Dashboard panels for exchange, strategy, order, and risk state;
- ignored-test closure for the v0.4 release claim.

Out of scope:

- real funds;
- production trading;
- production exchange connectivity;
- real account connectivity;
- real order submission;
- production Binance Spot or USDT-M parity;
- arbitrary user strategy loading;
- manual order entry;
- remote or multi-user Dashboard operation;
- Docker or prebuilt binary delivery as a v0.4.0 requirement.

## V04 Task Readiness

| Task | Scope | Evidence | PR | Status |
| --- | --- | --- | --- | --- |
| `V04-001` | Binance product boundary contract | `docs/rust-cutover/evidence/V04-001.md` | #279 | PASS |
| `V04-002` | Binance capability matrix | `docs/rust-cutover/evidence/V04-002.md` | #280 | PASS |
| `V04-003` | EMA / RSI strategy contracts | `docs/rust-cutover/evidence/V04-003.md` | #281 | PASS |
| `V04-004` | Strategy config DTO | `docs/rust-cutover/evidence/V04-004.md` | #282 | PASS |
| `V04-005` | Binance fixture market data replay | `docs/rust-cutover/evidence/V04-005.md` | #283 | PASS |
| `V04-006` | EMA smoke | `docs/rust-cutover/evidence/V04-006.md` | #286 | PASS |
| `V04-007` | RSI smoke | `docs/rust-cutover/evidence/V04-007.md` | #287 | PASS |
| `V04-008` | Mock order lifecycle | `docs/rust-cutover/evidence/V04-008.md` | #284 | PASS |
| `V04-009` | Risk rejection smoke | `docs/rust-cutover/evidence/V04-009.md` | #285 | PASS |
| `V04-010` | Dashboard exchange / strategy / order / risk panels | `docs/rust-cutover/evidence/V04-010.md` | #288 | PASS |
| `V04-011` | Ignored tests closure batch 2 | `docs/rust-cutover/evidence/V04-011.md` | #289 | PASS as scope closure |

## Evidence Map

| Readiness item | Evidence | Result |
| --- | --- | --- |
| Scope boundary says sandbox-only and no real funds | `V04-001`, scope doc | PASS |
| Binance Spot / USDT-M capability split is explicit | `V04-002` | PASS |
| EMA and RSI contracts exist before implementation smokes | `V04-003` | PASS |
| Strategy config DTO validates v0.4 fields | `V04-004` | PASS |
| Checked-in Binance fixture replay exists and is deterministic | `V04-005` | PASS |
| Mock order lifecycle exists for submit/accept/fill/cancel/reject evidence | `V04-008` | PASS |
| Risk rejection uses local deterministic halted-state smoke | `V04-009` | PASS |
| EMA smoke runs on the Binance fixture and emits stable output | `V04-006` | PASS |
| RSI smoke runs on the Binance fixture and emits stable output | `V04-007` | PASS |
| Dashboard renders exchange, strategy, order, and risk panels from evidence | `V04-010` | PASS |
| V04 product path does not depend on active ignored tests | `V04-011` | PASS as scope closure |

## Local Verification

| Command | Result | Summary |
| --- | --- | --- |
| `scripts/ai/verify_release.sh` | PASS | Full release verification passed. It ran full checks, release CLI binary build, Rust CLI product surface checks, Rust-only/Cython gates, v0.2 two-node smoke, v0.3 supervisor control smoke, and v0.3 dashboard smoke. |
| `scripts/ai/verify_full.sh` | PASS | Direct full verification passed: fast checks, clippy, workspace Rust tests, golden trace validation, and Rust docs. |
| `git diff --check` | PASS | No whitespace diff errors. |

Release smoke highlights:

- `v02_two_node_smoke status=ok`
- `v03_supervisor_control_smoke status=ok`
- `v03_dashboard_smoke status=ok`
- `verify_full complete`
- `verify_release complete`

## PASS / FAIL Decision

| Decision item | Result | Reason |
| --- | --- | --- |
| V04 queue evidence exists from `V04-001` through `V04-011` | PASS | All required evidence files exist and the corresponding PRs were merged. |
| Binance sandbox product foundation claim | PASS | Fixture replay, mock lifecycle, risk rejection, EMA/RSI smokes, and Dashboard panels are evidenced. |
| Production Binance trading readiness | FAIL | Explicitly out of scope; no real account, real funds, or real order submission is claimed. |
| V04-012 release verification | PASS | `verify_release.sh`, `verify_full.sh`, and `git diff --check` passed. |
| Publish tag or GitHub Release | FAIL | This task does not create a tag or publish a GitHub Release. |

Final v0.4 readiness decision: PASS for the scoped Binance Sandbox Product
Foundation.

## Behavior Impact

This report changes only release documentation and evidence.

No runtime behavior changed.

No trading-semantic behavior changed.

No adapter behavior changed.

## Public API Impact

No public API change.

No CLI command shape changed.

## Migration Note Status

No migration note is required. This is a readiness report, not a user API or
runtime contract change.

## Remaining Risks

- v0.4 does not claim production trading or real Binance connectivity.
- Binance USDT-M remains deferred unless later evidence records an approved
  sandbox scope.
- `cargo test -- --ignored` still fails on a non-v0.4 dYdX reconnect ignored
  test, as recorded in `V04-011`; this remains future adapter hardening, not
  v0.4 release evidence.
- Matching-engine, broad risk-engine, PostgreSQL cache, and live stress ignored
  tests remain scoped out for v0.4 rather than fixed.
- The Dashboard is local and evidence-backed; it is not a remote multi-user
  production trading cockpit.

## Next Step

After this report PR is merged and local verification is recorded, decide
separately whether to create a `v0.4.0` tag or publish a GitHub Release. This
report itself does not perform either action.
