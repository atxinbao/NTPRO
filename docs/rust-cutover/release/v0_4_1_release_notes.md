# NTPRO Rust-only v0.4.1 Release Notes

Date: 2026-06-13
Executor: Codex
Task ID: V041-005

## Release Identity

```text
Current source tag: ntpro-rust-only-v0.4.1
Capability: Binance Sandbox Product Foundation release-surface hardening
```

## Plain Chinese Summary

v0.4.1 是 v0.4.0 Binance sandbox 产品基础的补丁发布。

这次主要修“公开发布面”：README 指向 v0.4.1，新增一个明确的 v0.4 Binance
sandbox smoke 脚本，记录 GitHub hosted release gate PASS，并补齐 v0.4.1 readiness
report 和 release notes。

这不是 Binance 实盘版本。它不连接真实账户，不使用真实资金，不提交真实订单，也不
声明生产 Binance parity。

## Included

- `README.md` current-release wording aligned to `ntpro-rust-only-v0.4.1`.
- v0.4.1 patch scope contract.
- `scripts/ai/verify_v04_binance_sandbox.sh` as the explicit v0.4 Binance
  sandbox smoke gate.
- Hosted `Rust Cutover Release Gate` PASS evidence on
  `main@f79001646110bae5780b3e3b5949cc62086ba447`.
- v0.4.1 readiness report and release notes.

## Not Included

v0.4.1 does not claim these capabilities:

- production trading;
- real funds;
- real Binance account connectivity;
- real order submission;
- production Binance Spot parity;
- production Binance USDT-M parity;
- v0.5 local workflow artifacts;
- v0.6 optional testnet runtime;
- remote or multi-user Dashboard operation;
- prebuilt binary or Docker delivery as a v0.4.1 requirement.

## Validation Source

Primary readiness evidence:

- `docs/rust-cutover/scope/v0_4_1_binance_sandbox_release_surface_hardening.md`
- `docs/rust-cutover/evidence/V041-001.md`
- `docs/rust-cutover/evidence/V041-002.md`
- `docs/rust-cutover/evidence/V041-003.md`
- `docs/rust-cutover/evidence/V041-004.md`
- `docs/rust-cutover/evidence/V041-005.md`
- `docs/rust-cutover/release/v0_4_1_binance_sandbox_release_surface_hardening_readiness_report.md`

Verification:

- `scripts/ai/verify_v04_binance_sandbox.sh`: PASS
- Hosted `Rust Cutover Release Gate` run `27468867719`: PASS
- Hosted run URL:
  `https://github.com/atxinbao/NTPRO/actions/runs/27468867719`

## Boundary

```text
Binance sandbox-only
no real funds
no production trading
no real order submission
no new trading capability
```

## Migration Note Status

No migration note is required. v0.4.1 does not change runtime behavior, public
CLI shape, Rust library API, adapter behavior, persistence format, or trading
semantics.

