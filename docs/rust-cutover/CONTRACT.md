# Rust-Only Cutover Contract

## Objective

NautilusTrader must ship as a Rust-only product. Rust crates own runtime behavior, Rust CLI commands provide operational entrypoints, Rust examples demonstrate user workflows, and Rust documentation is the public usage path.

Python, PyO3, and Cython are not final product surfaces. They may exist temporarily only as comparison or migration surfaces until Rust-only parity and usability evidence is complete.

## Runtime Contract

```text
Rust crates = source of truth for runtime behavior
Rust CLI    = operational product entrypoint
Rust docs   = user-facing documentation surface
Python      = removed from final product surface
PyO3        = removed from final product surface
Cython      = removed from final product surface
```

## Compatibility Contract

For every scoped P0/P1 behavior:

```text
same input event stream -> same Rust output event stream
```

A difference is allowed only when:

1. a migration note explains the difference;
2. a scope decision approves it;
3. Rust tests assert the new behavior.

## Rust-Only Usability Contract

The cutover is not complete merely because Python/Cython files were deleted. The release candidate must prove a new user can use the project with Rust tooling only:

- build the workspace with Cargo;
- run the Nautilus CLI without Python;
- run at least one Rust backtest flow;
- construct or run at least one Rust live/sandbox node flow;
- exercise scoped official adapters through Rust tests or examples;
- read Rust docs or examples for the supported workflow.

## Evidence Contract

The following require golden trace or equivalent Rust integration evidence:

- order lifecycle;
- risk rejection;
- execution routing;
- position opening/changing/closing;
- PnL/accounting;
- cache/message-bus event ordering;
- backtest/live parity;
- adapter payload parsing.

## Build Contract

The final release path must build through Cargo only. It must not require `python/`, `nautilus_trader/`, `crates/pyo3`, maturin, Cython, `build.py`, or generated Python extension artifacts as product runtime/API/build surfaces.

## Removal Contract

Python, PyO3, and Cython are not compatibility surfaces. The cutover must port, replace, or remove every Python/PyO3/Cython-dependent runtime/test/build path. Old import paths may be handled only by documented Rust migration guidance or explicit breaking-change notes.

## Final Deletion Ordering

Inventory, planning, and non-destructive migration preparation may start before parity is complete. Hard deletion of remaining Python/PyO3/Cython product surfaces must wait until Rust CLI/API/example usability evidence, runtime parity evidence, Rust backtest/live evidence, and adapter parity evidence are complete. The final Rust-only state is enforced by `RREM-010`, `RREL-006`, and `scripts/ai/check_rust_only_runtime.sh`.
