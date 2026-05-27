# Definition of Done

The Rust-only cutover is complete only when all gates below are green.

## Gate 1 - Control Plane

- [ ] `AGENTS.md` exists and is current.
- [ ] Task graph exists.
- [ ] Lease protocol exists.
- [ ] PR evidence template exists.
- [ ] Verification scripts exist.

## Gate 2 - Inventory

- [ ] Cython inventory complete.
- [ ] Python/PyO3 product-surface inventory complete.
- [ ] v1/v2 parity matrix complete.
- [ ] Rust CLI/API/example entrypoint inventory complete.
- [ ] adapter parity matrix complete.
- [ ] build-path inventory complete.

## Gate 3 - Golden Traces

- [ ] market data traces pass.
- [ ] order lifecycle traces pass.
- [ ] risk traces pass.
- [ ] portfolio/PnL traces pass.
- [ ] backtest/live parity traces pass or scoped.

## Gate 4 - Rust Product Surface

- [ ] Rust workspace builds with Cargo.
- [ ] Rust CLI exposes supported `backtest`, `live` or `sandbox`, `data`, and diagnostic/help entrypoints.
- [ ] Rust CLI smoke tests pass.
- [ ] Rust backtest example smoke tests pass.
- [ ] Rust live/sandbox node smoke tests pass.
- [ ] Rust public API documentation is generated or validated.
- [ ] Rust user examples cover the supported workflow.

## Gate 5 - Python/PyO3/Cython Removal

- [ ] final product runtime does not require `python/`.
- [ ] final product runtime does not require `nautilus_trader/`.
- [ ] final product runtime does not require `crates/pyo3`.
- [ ] final product runtime does not require `build.py`, maturin, Cython, `*.pyx`, or `*.pxd`.
- [ ] all Python/PyO3/Cython source/build/test product paths are removed or replaced by Rust equivalents.
- [ ] final deletion occurs after Gate 6 and Gate 7 parity evidence, and before Gate 8 signoff.

## Gate 6 - Runtime Parity

- [ ] model parity complete.
- [ ] serialization parity complete.
- [ ] cache/message-bus parity complete.
- [ ] backtest parity complete.
- [ ] trading strategy/actor parity complete.
- [ ] risk/execution/portfolio parity complete.
- [ ] live node parity complete.

## Gate 7 - Adapter Parity

- [ ] scoped official adapters have parser fixtures.
- [ ] scoped execution adapters have lifecycle tests.
- [ ] reconnect/reconciliation behavior is tested or scoped.
- [ ] Rust adapter APIs and examples are checked.

## Gate 8 - Release

- [ ] Rust-only migration guide complete.
- [ ] release notes complete.
- [ ] final audit report generated.
- [ ] `scripts/ai/check_rust_only_runtime.sh` passes.
- [ ] human owner signoff complete.
