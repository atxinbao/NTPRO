# NTPRO Rust-only v0.6.1 Release Notes

Date: 2026-06-14
Executor: Codex
Release line: v0.6.1

## Release Identity

```text
Current source tag: ntpro-rust-only-v0.6.0
Hardening line: v0.6.1 contract/dashboard/CI hardening
Capability: offline hardening for Binance testnet dry-run runtime foundation
```

## Plain Chinese Summary

`v0.6.1` 是 `v0.6.0` 之后的 hardening 收口，不是新的联网能力发布。

这次把公开文案、run_id 规则、connectivity-probe 离线语义、Dashboard artifact
读取、manifest 子工件审计、PR-stage v0.6 smoke、writer/reader 共享 workflow
artifact contract 统一起来。

这不是 Binance testnet 实连版本。它不连接 Binance，不读取或保存真实 API key，不使用真实资金，
不提交真实订单，也不声明生产交易能力。

## Included

- README / roadmap / release wording aligned to the active v0.6.1 hardening
  track.
- One effective workflow `run_id` across CLI output and workflow artifacts.
- Offline-only connectivity-probe semantics:
  - `network_attempted=false`;
  - `testnet_connection=false`;
  - optional network intent is recorded as intent only.
- Dashboard workflow artifact browsing through an explicit workflow root even
  when the supervisor registry is missing.
- Dashboard health degradation when `manifest.artifacts[]` child artifacts are
  missing, invalid, empty, or schema-mismatched.
- PR-stage `v06-binance-testnet-dry-run-smoke` for relevant workflow/dashboard
  changes.
- Shared workflow artifact DTO/schema contract for CLI writer and Dashboard
  reader.
- v0.6.1 readiness report and release notes.

## Not Included

`v0.6.1` does not claim these capabilities:

- live Binance testnet network connection;
- real Binance testnet order submission;
- real account reconciliation;
- production Binance connectivity;
- real funds;
- production trading parity;
- Dashboard controls that start a probe or read credentials;
- v0.7 real read-only HTTP/WebSocket probe capability;
- prebuilt binary or Docker delivery as a release requirement.

## Validation Source

Primary readiness evidence:

- `docs/rust-cutover/release/v0_6_1_offline_hardening_readiness_report.md`
- `docs/rust-cutover/evidence/V061-001.md` through `docs/rust-cutover/evidence/V061-008.md`

Verification:

- `cargo fmt --check`: PASS
- `cargo test -p nautilus-cli workflow --lib`: PASS
- `cargo test -p nautilus-cli dashboard --lib`: PASS
- `cargo clippy -p nautilus-cli --lib --tests -- -D warnings`: PASS
- `scripts/ai/verify_v06_binance_testnet_dry_run.sh`: PASS
- `scripts/ai/verify_fast.sh`: PASS
- `git diff --check`: PASS

## Boundary

```text
v0.6.1 offline hardening only
no live network connection
no real funds
no real orders
no production trading
v0.7 real read-only probes remain future gated work
```

## Migration Note Status

No migration note is required. `v0.6.1` does not change user-facing artifact
paths, JSON field names, trading semantics, or public network behavior.
