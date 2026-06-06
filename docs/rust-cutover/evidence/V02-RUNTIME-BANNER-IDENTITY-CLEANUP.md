# V02 Runtime Banner Identity Cleanup Evidence

Date: 2026-06-06
Executor: Codex

## Task

Local release blocker cleanup for v0.2 readiness.

## Goal

Clean the runtime startup banner so NTPRO no longer presents the upstream
NautilusTrader product identity during Rust-only runtime startup.

## Why This Was Needed

During the second v0.2 release verification attempt on `main@1febff2505`,
`scripts/ai/verify_release.sh` reached the workspace test phase and repeatedly
printed the old startup banner:

- `NAUTILUS TRADER - Automated Algorithmic Trading Platform`
- `by Nautech Systems Pty Ltd.`
- `nautilus_trader: 1.228.0`

The tests were passing, but the public runtime output conflicted with NTPRO's
Rust-only product identity. This is a release-readiness issue, not a trading
semantic issue.

## Files Changed

- `crates/common/src/logging/headers.rs`

## Change Summary

- Replaced the runtime startup title with `NTPRO - Rust-only Trading Engine Workspace`.
- Replaced the public banner author line with a lineage-safe Rust-only cutover line.
- Kept the copyright/license lineage in the source file header.
- Replaced the runtime version label from `nautilus_trader` to `ntpro`.
- Added a unit test to ensure the public runtime banner constants do not regress to
  the old upstream product identity.

## Commands Run

- `cargo fmt`
- `cargo test -p nautilus-common --lib runtime_banner_uses_ntpro_product_identity -- --nocapture`
  - Result: passed, 1 test.
- `cargo test -p nautilus-live --test golden_trace_live_sandbox rust_sandbox_live_node_replays_lifecycle_golden_trace -- --nocapture`
  - Result: passed, 1 test.
  - Runtime output now prints `NTPRO - Rust-only Trading Engine Workspace` and
    `ntpro: 1.228.0`.
- `scripts/ai/verify_fast.sh`
  - Result: passed; this is fast smoke only.
- `cargo check -p nautilus-common`
  - Result: passed.
- `git diff --check`
  - Result: passed.
- `rg -n "NAUTILUS TRADER|Automated Algorithmic Trading Platform|by Nautech Systems Pty Ltd\.|nautilus_trader:" crates README.md docs Cargo.toml`
  - Result: classified residual matches only.
  - Reasonable residuals:
    - this evidence file documents the old banner observed before cleanup;
    - `DRG-003.md` records the previous release-readiness concern;
    - the new unit test asserts old public banner strings do not return;
    - `logging/config.rs` keeps `nautilus_trader::...` module-name parsing test
      data, not runtime product banner output.

## Behavior Impact

Only runtime log/banner text changes. No order routing, matching, risk,
portfolio, adapter, serialization, or persistence semantics change.

## Public API Impact

No Rust API signature change.

## Migration Note

No user migration is required. Users will see NTPRO-branded runtime startup logs
instead of the old upstream product banner.

## Rollback Plan

Revert this PR to restore the previous banner text.
