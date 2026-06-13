# NTPRO v0.6.0 Binance Testnet Dry-Run Readiness Report

Date: 2026-06-13
Executor: Codex
Milestone: v0.6.0 Binance testnet dry-run foundation
Decision: PASS for scoped offline dry-run foundation

## Plain Chinese Summary

v0.6.0 的产品口径是：建立 Binance testnet runtime foundation 的离线 dry-run 基础。

大白话说：现在可以用 Rust CLI 运行 `binance-testnet` workflow，读取 checked-in
testnet 配置，生成 credential policy、connectivity probe、order lifecycle、
reconciliation、summary、events 和 manifest artifact。Dashboard 也能只读展示这些
testnet dry-run 状态。

这不是 Binance testnet 实连版本。它不连接 Binance，不读取或保存真实 API key，不使用真实资金，
不提交真实订单，也不声明生产交易能力。

## Scope Claim

```text
Binance testnet dry-run runtime foundation
```

In scope:

- `nautilus workflow run --workflow binance-testnet --mode dry-run`;
- checked-in testnet dry-run config;
- env-var-only credential policy artifact;
- offline connectivity probe artifact;
- dry-run order lifecycle artifact;
- artifact-only reconciliation artifact;
- Dashboard read-only testnet workflow surface;
- explicit v0.6 smoke;
- release gate integration.

Out of scope:

- live Binance testnet network connection;
- real Binance testnet order submission;
- real account reconciliation;
- production Binance connectivity;
- real funds;
- production trading parity;
- tag or GitHub Release creation.

## V06 Task Readiness

| Task | Scope | Evidence | Status |
| --- | --- | --- | --- |
| `V06-001` | Binance Testnet Runtime Foundation | `docs/rust-cutover/evidence/V06-001.md` | PASS |
| `V06-002` | testnet credential policy | `docs/rust-cutover/evidence/V06-002.md` | PASS |
| `V06-003` | dry-run and optional testnet runtime | `docs/rust-cutover/evidence/V06-003.md` | PASS |
| `V06-004` | testnet CLI and config contract | `docs/rust-cutover/evidence/V06-004.md` | PASS |
| `V06-005` | testnet connectivity validation probe | `docs/rust-cutover/evidence/V06-005.md` | PASS |
| `V06-006` | optional runtime wiring | `docs/rust-cutover/evidence/V06-006.md` | PASS |
| `V06-007` | order lifecycle and reconciliation evidence | `docs/rust-cutover/evidence/V06-007.md` | PASS |
| `V06-008` | Dashboard testnet runtime surface | `docs/rust-cutover/evidence/V06-008.md` | PASS |
| `V06-009` | explicit v0.6 smoke gate | `docs/rust-cutover/evidence/V06-009.md` | PASS |
| `V06-010` | user docs | `docs/rust-cutover/evidence/V06-010.md` | PASS |
| `V06-011` | release gate integration | `docs/rust-cutover/evidence/V06-011.md` | PASS |
| `V06-012` | readiness report | `docs/rust-cutover/evidence/V06-012.md` | PASS |

## Verification

| Command | Result | Summary |
| --- | --- | --- |
| `cargo test -p nautilus-cli workflow --lib` | PASS | Workflow parser, V05 compatibility, V06 dry-run artifacts, and missing-config rejection passed. |
| `cargo test -p nautilus-cli dashboard --lib` | PASS | Dashboard testnet workflow manifest surface passed. |
| `scripts/ai/verify_v06_binance_testnet_dry_run.sh` | PASS | CLI writes and validates 9 dry-run artifacts and 7 events. |
| `scripts/ai/verify_release.sh v06-binance-testnet-dry-run-smoke` | PASS | Release verifier runs v0.6 smoke with release CLI binary. |
| `scripts/ai/verify_fast.sh` | PASS | Toolchain and formatting smoke passed. |
| `cargo clippy -p nautilus-cli --lib --tests -- -D warnings` | PASS | CLI lint gate passed locally. |

## PASS / FAIL Decision

| Decision item | Result | Reason |
| --- | --- | --- |
| Offline Binance testnet dry-run foundation | PASS | CLI/config/artifacts/smoke evidence exists. |
| Credential policy | PASS | Config records env var names only; artifacts record `values_recorded=false`. |
| Connectivity validation probe | PASS | Offline probe validates configured adapter URLs and records `network_attempted=false`. |
| Order lifecycle and reconciliation artifacts | PASS | Dry-run lifecycle and artifact-only reconciliation are generated and validated. |
| Dashboard testnet dry-run surface | PASS | Dashboard reads and renders testnet dry-run fields. |
| Live Binance testnet connection | FAIL | Explicitly out of scope for v0.6. |
| Real Binance testnet orders | FAIL | Explicitly out of scope; no real orders are submitted. |
| Production trading readiness | FAIL | Explicitly out of scope. |

Final v0.6.0 readiness decision: PASS for scoped offline Binance testnet dry-run
foundation.

## Behavior Impact

Adds an offline Binance testnet workflow artifact path. No network connection,
trading semantic behavior, adapter execution behavior, or persistence format is
changed.

## Public API Impact

The Rust CLI `workflow run` command adds:

- `--workflow binance-testnet`;
- `--mode dry-run|connectivity-probe`;
- `--config`;
- `--allow-testnet-network`.

The local Dashboard workflow artifact DTO adds read-only testnet and network
boundary fields.

## Migration Note Status

No migration note is required. This is additive Rust-only dry-run functionality.

## Remaining Risks

- Real Binance testnet connectivity remains unproven by design.
- Credential value handling beyond env-var naming remains out of scope.
- Real testnet order lifecycle and real account reconciliation require future
  gated tasks.

## Next Step

If the project wants real Binance testnet connectivity, create a new scoped task
that requires explicit credentials policy, network opt-in, recorded fixture or
sandbox evidence, and a separate release gate.
