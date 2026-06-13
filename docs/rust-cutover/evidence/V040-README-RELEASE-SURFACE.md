# V040 README Release Surface Evidence

Date: 2026-06-13
Executor: Codex
Task ID: V040-README-RELEASE-SURFACE

## Plain Chinese Summary

这次只修 README 的公开发布口径。`ntpro-rust-only-v0.4.0` 已经发布，
README 不能继续写当前里程碑是 v0.3.0，也不能把 v0.4 说成“规划中”。

大白话说：首页现在会告诉读者当前正式发布点是 v0.4.0，能力边界是 Binance
sandbox 产品基础；同时明确这不是 Binance 实盘，不连接真实账户，不使用真实资金，
也不提交真实订单。

## Files Changed

- `README.md`
  - Updates the current source tag from `ntpro-rust-only-v0.3.0` to
    `ntpro-rust-only-v0.4.0`.
  - Updates the current capability from Local Supervisor Control Console to
    Binance Sandbox Product Foundation.
  - Replaces the planned v0.4 boundary wording with the current v0.4.0 Binance
    sandbox boundary.
  - Adds the v0.4.0 readiness report to the release document list.
- `docs/rust-cutover/evidence/V040-README-RELEASE-SURFACE.md`
  - Records this docs-only release-surface cleanup.

## Commands Run

```bash
scripts/ai/verify_fast.sh
```

Result: PASS. The script confirmed Cargo/Rust `1.95.0`, ran `cargo fmt
--check`, and reported that workspace `cargo check`, clippy, and release gates
are outside the fast-smoke default.

```bash
rg -n "ntpro-rust-only-v0.3.0|planned v0.4|current v0.3.0|ntpro-rust-only-v0.4.0|Binance sandbox-only|no real funds|no production trading" README.md docs/rust-cutover/evidence/V040-README-RELEASE-SURFACE.md
```

Result: PASS. README now points at `ntpro-rust-only-v0.4.0`, describes
`Binance sandbox-only`, and keeps `no real funds` / `no production trading`.
Remaining `ntpro-rust-only-v0.3.0` matches are only in this evidence file as
change-history text.

```bash
rg -n "current.*v0\\.3\\.0|planned v0\\.4|ntpro-rust-only-v0\\.3\\.0 is the current" README.md || true
```

Result: PASS. No README matches remain for stale current-v0.3.0 or planned-v0.4
release-surface wording.

```bash
git diff --check
```

Result: PASS.

## Behavior Impact

No runtime behavior changed.

No trading-semantic behavior changed.

No adapter behavior changed.

## Public API Impact

No public API change.

No CLI command shape changed.

## Migration Note Status

No migration note is required. This is a public documentation cleanup after the
v0.4.0 GitHub Release.

## Rollback Plan

Revert:

- `README.md`
- `docs/rust-cutover/evidence/V040-README-RELEASE-SURFACE.md`

Rollback would restore the previous README wording, but that would again make
the public milestone conflict with the published v0.4.0 release.
