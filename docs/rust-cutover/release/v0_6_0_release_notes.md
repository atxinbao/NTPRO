# NTPRO Rust-only v0.6.0 Release Notes

Date: 2026-06-14
Executor: Codex
Release line: v0.6.0

## Release Identity

```text
Current source tag: ntpro-rust-only-v0.6.0
Capability: Binance testnet dry-run runtime foundation
```

## Plain Chinese Summary

`v0.6.0` 是当前从 `v0.4.1` 继续向前的下一次正式发布。

这次公开发布的产品口径是：提供一个 **离线、可审计、Rust-only 的 Binance
testnet dry-run runtime foundation**。用户可以通过 Rust CLI 运行
`binance-testnet` workflow，读取 checked-in 配置，生成 credential policy、
connectivity probe、order lifecycle、reconciliation、summary、events 和
manifest artifact，并在 Dashboard 里只读查看这些状态。

`v0.5.0` 没有单独作为 public GitHub Release 发布。它作为
**local Binance sandbox workflow artifacts 基础层** 被 `v0.6.0` 吸收，并通过
`V05 compatibility` 验证继续保留在 `v0.6.0` 的正式发布树里。

这不是 Binance testnet 实连版本。它不连接 Binance，不读取或保存真实 API key，
不使用真实资金，不提交真实订单，也不声明生产交易能力。

## Included

- `README.md` current-release wording aligned to `ntpro-rust-only-v0.6.0`.
- `v0.5` local Binance sandbox workflow artifacts absorbed into the released
  source tree, without a separate public `v0.5.0` tag/release.
- Offline Binance testnet dry-run workflow foundation.
- Checked-in testnet dry-run config and env-var-only credential policy.
- Offline connectivity probe artifact.
- Dry-run order lifecycle and artifact-only reconciliation output.
- Dashboard read-only testnet workflow surface.
- Explicit `v0.5` and `v0.6` smoke gates in the release verifier.
- `v0.6.0` readiness report and release notes.

## Not Included

`v0.6.0` does not claim these capabilities:

- live Binance testnet network connection;
- real Binance testnet order submission;
- real account reconciliation;
- production Binance connectivity;
- real funds;
- production trading parity;
- remote or multi-user Dashboard operation;
- prebuilt binary or Docker delivery as a release requirement.

## Validation Source

Primary readiness evidence:

- `docs/rust-cutover/release/v0_5_0_workflow_artifacts_readiness_report.md`
- `docs/rust-cutover/release/v0_6_0_binance_testnet_dry_run_readiness_report.md`
- `docs/rust-cutover/evidence/V05-001.md` through `docs/rust-cutover/evidence/V05-011.md`
- `docs/rust-cutover/evidence/V06-001.md` through `docs/rust-cutover/evidence/V06-012.md`

Verification:

- `cargo test -p nautilus-cli workflow --lib`: PASS
- `cargo test -p nautilus-cli dashboard --lib`: PASS
- `scripts/ai/verify_v05_workflow_artifacts.sh`: PASS
- `scripts/ai/verify_v06_binance_testnet_dry_run.sh`: PASS
- `scripts/ai/verify_release.sh v05-workflow-artifacts-smoke v06-binance-testnet-dry-run-smoke`: PASS
- `scripts/ai/verify_fast.sh`: PASS
- Hosted `Rust Cutover Release Gate`: required on the final release commit

## Boundary

```text
Binance testnet dry-run only
no live network connection
no real funds
no real orders
no production trading
v0.5 absorbed, not separately released
```

## Migration Note Status

No migration note is required. `v0.6.0` is an additive Rust-only workflow and
artifact release. It does not change production trading semantics or introduce
real testnet execution behavior.
