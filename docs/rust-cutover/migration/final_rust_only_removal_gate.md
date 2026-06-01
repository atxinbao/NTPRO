# RREM-010 Final Rust-Only Removal Gate

Date: 2026-06-01
Executor: Codex
Task ID: RREM-010

## Scope

This document records the final Rust-only removal gate decision for the current
repository state. It does not delete, move, skip, or weaken Python, PyO3,
Cython, build, runtime, adapter, test, or documentation files.

## Final Gate Decision

The final Rust-only removal gate is blocked.

NTPRO cannot truthfully be declared Rust-only yet because active Python, PyO3,
Cython, runtime/API/build surfaces remain in the repository.

## Required Gate Results

| Gate command | Result | Decision |
| --- | --- | --- |
| `scripts/ai/verify_full.sh` | Timed out after 120 seconds with Rust 1.95.0 while running workspace tests | Not green. |
| `scripts/ai/check_rust_only_runtime.sh` | Failed | Not green. |
| `scripts/ai/check_cython_removed.sh` | Failed | Not green. |

## Remaining Product Surfaces

| Surface | Current result |
| --- | --- |
| `python/` | still exists |
| `nautilus_trader/` | still exists |
| `crates/pyo3/` | still exists |
| `build.py` | still exists |
| `crates/**/src/python` directories | 36 remain |
| Cython `.pyx` / `.pxd` files | 243 remain |

## Final Blockers

The following blockers must be cleared before RREM-010 can be considered a
passing release gate:

- Python package surface removal or explicit non-product migration archive.
- Top-level `nautilus_trader/` product runtime/API removal or replacement.
- PyO3 aggregator and per-crate binding removal after Rust replacement evidence.
- Cython source/interface and active build cleanup.
- Cargo, Python package, and build metadata cleanup for PyO3/Cython/maturin.
- Full verification completion under the pinned Rust toolchain.
- Rust-only runtime and Cython-removed scripts passing without ignored product
  paths.
- User-facing migration notes for removed Python imports, examples, install
  commands, and build workflows.

## Release Recommendation

Do not release as Rust-only from this state.

The correct next step is to keep removal work in staged, auditable slices:

1. Port or explicitly scope remaining Python/PyO3/Cython behavior.
2. Remove Cython source/build paths only after parity evidence is green.
3. Remove PyO3 bindings and Cargo metadata only after Python import migration
   notes and Rust replacement paths are ready.
4. Remove Python package surfaces only after product docs and examples are
   Rust-only.
5. Re-run RREM-009/RREM-010 gates after each destructive removal slice.
