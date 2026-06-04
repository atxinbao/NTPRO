# Post-release How-to Rust Links Cleanup Evidence

- Date: 2026-06-05
- Executor: Codex
- Local task name: `POST-RELEASE-HOW-TO-RUST-LINKS-CLEANUP`
- Formal task file: not present in the Shrimp queue; this is a scoped
  post-release public docs cleanup.

## Goal

Update Rust how-to example links so user-facing NTPRO documentation points at
the current NTPRO repository paths instead of the upstream NautilusTrader
`develop` branch.

## Scope

Changed:

- `docs/how_to/run_rust_backtest.md`
- `docs/how_to/write_rust_actor.md`
- `docs/how_to/write_rust_strategy.md`

Not changed:

- runtime code;
- examples;
- Cargo feature flags;
- integration docs;
- deep concept docs;
- release docs.

## Changes

- Replaced two backtest example source links with local repository paths:
  - `../../crates/backtest/examples/engine_ema_cross.rs`
  - `../../crates/backtest/examples/node_ema_cross.rs`
- Replaced actor example link with local repository path:
  - `../../crates/trading/src/examples/actors/imbalance`
- Replaced strategy example links with local repository paths:
  - `../../crates/trading/src/examples/strategies/ema_cross`
  - `../../crates/trading/src/examples/strategies/grid_mm`

## Validation

| Command | Result | Notes |
|---------|--------|-------|
| `rg -n "github.com/nautechsystems/nautilus_trader/tree/develop/crates|github.com/nautechsystems/nautilus_trader/blob/develop/crates|nautechsystems/nautilus_trader" docs/how_to/run_rust_backtest.md docs/how_to/write_rust_actor.md docs/how_to/write_rust_strategy.md` | passed | No matches in the scoped files. |
| Relative path existence check | passed | All five new local paths resolve to existing files or directories. |
| `scripts/ai/check_rust_only_runtime.sh` | passed | Reported `rust-only-runtime: ok`. |
| `scripts/ai/check_cython_removed.sh` | passed | Reported `cython-removed: ok`. |
| `scripts/ai/verify_fast.sh` | passed | Used cargo/rustc 1.95.0; workspace cargo check and clippy skipped by fast-mode defaults. |
| `git diff --check` | passed | No whitespace errors. |

## Behavior Impact

No runtime behavior changed. This is a Markdown link cleanup only.

## Public API Impact

No public API changed.

## Migration Note Status

This PR supports the existing Rust-only migration posture by ensuring Rust
how-to docs link to NTPRO-owned examples instead of upstream NautilusTrader
example paths.

## Rollback Plan

Revert this PR to restore the old upstream links. No runtime, API, dependency,
or data migration is required.
