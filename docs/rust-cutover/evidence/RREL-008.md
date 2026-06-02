# RREL-008 Evidence - Rust-Only Completion Gate

Date: 2026-06-02
Executor: Codex
Task ID: RREL-008

## Summary

RREL-008 did not mark the Rust-only cutover complete.

The latest verification moved the project forward: the Rust-only runtime
surface check and final Cython removal check now pass, and the standard golden
trace command passes its schema plus built-in Rust replay harnesses. The final
release gate is still blocked because `scripts/ai/verify_release.sh` enables the
strict final replay gate and `scripts/ai/run_golden_traces.sh` requires a
`GOLDEN_TRACE_REPLAY_COMMAND` for that mode. Owner signoff is also still
pending.

## Files Changed

- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREL-008.json`
- `docs/rust-cutover/evidence/RREL-008.md`
- `docs/rust-cutover/golden_trace/GATE_EVIDENCE.md`
- `docs/rust-cutover/release/final_release_verification.md`
- `docs/rust-cutover/release/human_owner_signoff_packet.md`
- `docs/rust-cutover/release/final_completion_report.md`
- `docs/rust-cutover/release/release_candidate_tag_plan.md`
- `crates/model/src/orders/mod.rs`
- `crates/model/src/position.rs`
- `crates/model/src/types/money.rs`
- `crates/adapters/*/test_data/rust_adapter_parity_closure.json`
- `crates/adapters/dydx/src/grpc/client.rs`
- `tests/golden/backtest_live_semantic_parity_schema.jsonl`

## Commands Run

```bash
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 \
  PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
  scripts/ai/verify_release.sh

PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
  scripts/ai/check_rust_only_runtime.sh

PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
  scripts/ai/check_cython_removed.sh

PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
  scripts/ai/run_golden_traces.sh

PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
  cargo clippy -p nautilus-model --lib --tests --features high-precision,ffi -- -D warnings

PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
  cargo test -p nautilus-model --lib orders

PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
  cargo test -p nautilus-model --lib position

PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
  cargo test -p nautilus-dydx --lib

CARGO_INCREMENTAL=0 \
  PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" \
  cargo clippy -p nautilus-dydx --lib --tests --features high-precision -- -D warnings
```

## Command Results

- `scripts/ai/verify_release.sh`: failed after completing full workspace
  clippy/tests and the log-global test slices. It stopped at final golden trace
  validation with `GOLDEN_TRACE_REPLAY_COMMAND is required for final release
  replay gate`.
- `scripts/ai/check_rust_only_runtime.sh`: passed.
- `scripts/ai/check_cython_removed.sh`: passed.
- `scripts/ai/run_golden_traces.sh`: passed. It validated all golden trace
  JSONL fixtures and ran the Rust schema, cache/msgbus, backtest, live sandbox,
  and OKX adapter payload replay harnesses.
- Targeted model clippy/tests passed after small release-gate fixes.
- Targeted dYdX tests and clippy passed after making an invalid-URL test
  independent of local port state.

## Release-Gate Fixes Made

- Removed clippy blockers in `nautilus-model` by turning no-op order handlers
  into associated functions, removing an unused receiver from average-price
  calculation, and replacing an unchecked precision cast with `i32::from`.
- Updated stale adapter parity closure manifests so they no longer reference
  deleted `removed_legacy_bridge` evidence paths.
- Updated the backtest/live semantic parity fixture to use the current
  `rust_only_surface` field.
- Made the dYdX invalid fallback URL test use malformed URLs instead of a local
  port that can depend on host state.

## Tests Added Or Updated

No new test files were added. Existing release-gate tests and fixture manifests
were adjusted so the current Rust-only workspace can be verified.

## Behavior Impact

No trading semantics, order routing, adapter protocol behavior, or public
runtime API was intentionally changed. The code changes are release-gate
stability and lint fixes.

## Public API Impact

None.

## Migration Note Status

No migration note is required because no public API changed and RREL-008 did not
mark the release complete.

## Completion Decision

RREL-008 is blocked and must remain unmerged as a completion decision until:

1. `scripts/ai/verify_release.sh` passes in final mode.
2. The final golden trace replay command or equivalent release replay contract
   is wired and documented.
3. Human owner signoff is explicitly granted.
4. The release gatekeeper and control/scope review approve completion.

## Rollback Plan

Revert this PR to remove the RREL-008 evidence refresh and release-gate
stability fixes. The repository will return to the previous state where final
release verification still failed, but with less specific blocker evidence.
