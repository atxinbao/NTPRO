# Rust API Entrypoints

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-013

## Purpose

This document records the Rust-facing public entrypoints that are currently
usable for NTPRO cutover work. It complements the CLI contracts by pointing
owners to the Cargo packages, Rust examples, and generated Rust docs that can
be used without Python, PyO3, or Cython as the product path.

## Documentation Command

Generate the Rust API documentation with:

```bash
cargo doc --workspace --exclude nautilus-pyo3 --features arrow,ffi,high-precision,streaming,defi --no-deps
```

The generated documentation root is:

```text
target/doc/nautilus_trader/index.html
```

For product-surface work, the first pages to inspect are:

- `target/doc/nautilus_cli/index.html`
- `target/doc/nautilus_backtest/index.html`
- `target/doc/nautilus_live/index.html`
- `target/doc/nautilus_data/index.html`
- `target/doc/nautilus_sandbox/index.html`
- `target/doc/nautilus_trading/index.html`

## CLI Entrypoints

The CLI package is the user-facing operational entrypoint:

```text
package: nautilus-cli
binary:  nautilus
source:  crates/cli/src/bin/cli.rs
options: crates/cli/src/opt.rs
```

The current Rust-first command contracts are documented in:

- `docs/rust-cutover/product/RUST_CLI_CONTRACT.md`
- `docs/rust-cutover/product/BACKTEST_CLI_CONTRACT.md`
- `docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md`
- `docs/rust-cutover/product/DATA_CATALOG_CLI_CONTRACT.md`
- `docs/rust-cutover/product/CONFIG_VALIDATION_CLI_CONTRACT.md`

Current help-level entrypoints:

```bash
cargo run -q -p nautilus-cli -- backtest --help
cargo run -q -p nautilus-cli -- sandbox --help
cargo run -q -p nautilus-cli -- live --help
cargo run -q -p nautilus-cli -- data --help
cargo run -q -p nautilus-cli -- config --help
```

Runtime execution for backtest, sandbox, live, data, and config commands still
returns explicit blockers until later config-parser and runtime wiring tasks.

## Backtest Entrypoints

Primary package:

```text
package: nautilus-backtest
crate:   nautilus_backtest
docs:    target/doc/nautilus_backtest/index.html
```

Runnable Cargo smoke:

```bash
cargo run -p nautilus-backtest --features examples --example engine-ema-cross
```

Reference source:

- `crates/backtest/examples/engine_ema_cross.rs`
- `examples/rust/backtest/README.md`
- `docs/rust-cutover/evidence/RPROD-011.md`

## Sandbox And Live Entrypoints

Primary packages:

```text
package: nautilus-live
crate:   nautilus_live
docs:    target/doc/nautilus_live/index.html

package: nautilus-sandbox
crate:   nautilus_sandbox
docs:    target/doc/nautilus_sandbox/index.html
```

Runnable Cargo smoke:

```bash
cargo run -p nautilus-live --no-default-features --features node --example sandbox-node-smoke
```

Reference source:

- `crates/live/examples/sandbox_node_smoke.rs`
- `examples/rust/sandbox/README.md`
- `docs/rust-cutover/evidence/RPROD-012.md`

The sandbox smoke constructs a Rust `LiveNode` in `Sandbox` mode and does not
connect to a production venue.

## Data And Config Entrypoints

Primary packages:

```text
package: nautilus-data
crate:   nautilus_data
docs:    target/doc/nautilus_data/index.html

package: nautilus-model
crate:   nautilus_model
docs:    target/doc/nautilus_model/index.html
```

Current command contracts:

- `examples/rust/data/README.md`
- `examples/rust/config/README.md`
- `docs/rust-cutover/evidence/RPROD-007.md`
- `docs/rust-cutover/evidence/RPROD-008.md`
- `docs/rust-cutover/evidence/RPROD-009.md`

These surfaces are help-level and validation-contract level until the later
config-parser tasks connect config files to Rust model structs.

## Runtime Crate Entrypoints

Core runtime crates with public docs:

- `nautilus_common`: actor, cache, clock, message bus, and shared runtime
  infrastructure.
- `nautilus_core`: shared identifiers, time, correctness helpers, and core
  primitives.
- `nautilus_model`: domain model types for instruments, orders, events, and
  accounting data.
- `nautilus_trading`: actors, strategies, and execution algorithms.
- `nautilus_backtest`: historical simulation runtime.
- `nautilus_live`: live and sandbox node lifecycle runtime.
- `nautilus_sandbox`: simulated execution adapter for sandbox workflows.
- `nautilus_execution`: execution engine and matching components.
- `nautilus_risk`: risk checks and sizing.
- `nautilus_portfolio`: portfolio and account-state tracking.

## Boundary Rules

- Use Rust crates, Cargo examples, and generated Rust docs as the public product
  path for RPROD tasks.
- Do not use Python examples to satisfy Rust product-surface acceptance.
- Do not require PyO3, Cython, maturin, `build.py`, or generated Python
  extension artifacts for these entrypoints.
- Keep deletion of Python, PyO3, and Cython surfaces gated by the later RREM
  and release tasks.
