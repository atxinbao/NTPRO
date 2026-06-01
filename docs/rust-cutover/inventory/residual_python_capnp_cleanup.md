# Residual Python and Cap'n Proto Cleanup Inventory

Date: 2026-06-02
Executor: Codex
Task ID: RREM-015

## Summary

This audit separates three cleanup surfaces after RREM-014:

- Cap'n Proto: a standalone Rust serialization feature, removed by RREM-015.
- Python: remaining files are mostly historical tests and local automation
  scripts, not `python/` or `nautilus_trader/` product package directories.
- C/C++: no tracked `.c`, `.h`, `.cpp`, or `.hpp` source files remain in the
  current repository state.

RREL-008 remains paused. This task does not mark the Rust-only release complete.

## Cap'n Proto Decision

Cap'n Proto was not a Python residue. It was an optional Rust serialization
feature with its own Cargo feature, schema files, generated Rust files,
conversion modules, tests, benches, CI setup, Makefile commands, and public
documentation.

RREM-015 removes it as a product surface because NTPRO does not need this wire
format for the Rust-only cutover. The remaining supported serialization paths
are Arrow, Parquet/catalog storage, JSON, MsgPack, and SBE.

Removed surfaces:

- `crates/serialization/schemas/capnp/**`
- `crates/serialization/generated/capnp/**`
- `crates/serialization/src/capnp/**`
- `crates/common/src/serialization/capnp/**`
- Cap'n Proto-only tests and benches in `crates/serialization`
- `scripts/install-capnp.sh`
- `scripts/regen-capnp.sh`
- `.github/actions/install-capnp/**`
- `make check-capnp-schemas`
- `make regen-capnp`
- CI invocations of `EXTRA_FEATURES="capnp"` and Cap'n Proto cache inputs

Dependency boundary:

- `capnpc` is no longer present in `cargo tree --workspace --all-features`.
- `capnp` still appears as a transitive dependency of `hypersync-client` through
  the optional blockchain adapter path. That is not the removed
  `nautilus-serialization` Cap'n Proto product feature, so the existing audit
  waiver for the transitive advisory remains valid.

## Remaining Python Files

Tracked Python file count before this task's test cleanup: `540`.

By top-level surface:

| Surface | Count | Decision |
| --- | ---: | --- |
| `tests/**` | 534 | Do not delete in RREM-015. These files may still encode behavior regression coverage and need module-by-module Rust replacement or explicit scope removal. |
| `scripts/**` | 6 | Do not delete in RREM-015. These are local automation/control scripts currently used by the cutover workflow. |

By test family:

| Surface | Count |
| --- | ---: |
| `tests/unit_tests` | 284 |
| `tests/integration_tests` | 201 |
| `tests/performance_tests` | 24 |
| `tests/mem_leak_tests` | 18 |
| `tests/acceptance_tests` | 3 |
| `tests/docs_tests` | 2 |
| `tests/conftest.py` and package markers | 2 |

## Remaining Python-Facing Docs and Config

Public docs/config still contain Python, PyO3, or Cython references. RREM-015
does not rewrite all of them because this task focuses on Cap'n Proto removal
and audit. The next cleanup task should decide whether each file is deleted,
rewritten as Rust-only, or moved into historical migration notes.

Current file groups with Python-facing references:

| Group | File count |
| --- | ---: |
| `.github/**` | 22 |
| `docs/developer_guide/**` | 13 |
| `docs/tutorials/**` | 9 |
| `docs/getting_started/**` | 2 |
| `docs/api_reference/**` | 2 |
| `docs/how_to/**` | 1 |
| `README.md` | 1 |
| `Cargo.toml` | 1 |
| `pyproject.toml` | 1 |
| `.pre-commit-config.yaml` | 1 |

Representative next cleanup targets:

- `README.md`
- `docs/getting_started/installation.md`
- `docs/how_to/configure_live_trading.md`
- `docs/developer_guide/python.md`
- `docs/developer_guide/ffi.md`
- `docs/developer_guide/spec_data_testing.md`
- `docs/tutorials/*.md`
- `.github/workflows/*.yml`
- `.github/actions/common-wheel-build/action.yml`
- `pyproject.toml`
- `.pre-commit-config.yaml`

## Remaining Rust PyO3 and Cython References

The strict Rust-only gates are not green after RREM-015. This is expected and
is recorded as follow-up cleanup, not as a Cap'n Proto blocker.

Validation results:

| Command | Result | Meaning |
| --- | --- | --- |
| `scripts/ai/check_cython_removed.sh` | Pass | `.pyx`/`.pxd` sources and old package build paths are removed. |
| `scripts/ai/check_no_cython_runtime.sh` | Fail | Rust crates still contain Cython generation/config/parity references. |
| `scripts/ai/check_rust_only_runtime.sh` | Fail | Rust crates still contain broad PyO3 annotations, `python` feature references, and Cython references. |

Representative active Rust cleanup targets:

- `crates/core/build.rs`, `crates/backtest/build.rs`,
  `crates/common/build.rs`, and `crates/model/build.rs` still mention Cython
  header generation.
- `crates/*/cbindgen_cython.toml` files still exist.
- Many Rust modules still contain `pyo3::pyclass`,
  `pyo3_stub_gen::derive::*`, `pyo3::Python`, or `python` feature docs.
- Several Rust comments still reference Cython parity. These should be triaged
  separately from executable Cython build references.

Next step recommendation:

1. Remove active Cython header generation and `cbindgen_cython.toml` files.
2. Split PyO3 annotation removal by crate family, starting with small crates.
3. Update Rust docs and comments only after behavior parity references are no
   longer needed as migration evidence.

## C/C++ Check

Tracked C/C++ source file count:

```text
git ls-files '*.c' '*.h' '*.cpp' '*.hpp' -> 0
```

No C/C++ cleanup PR is needed from the current tracked tree. If GitHub language
statistics still show C after this PR, that is likely a cached or historical
Linguist calculation rather than live tracked source code.

## Do Not Delete Yet

Do not delete `tests/**/*.py` as a bulk operation. The safer next step is a
dedicated task that groups tests by runtime area:

1. model/core/accounting/risk value tests;
2. backtest/live/data/execution tests;
3. adapter integration tests;
4. performance and memory tests;
5. docs/acceptance tests.

Each group needs one of:

- Rust test replacement exists and is linked;
- behavior is intentionally dropped with a scope decision;
- test is still needed as migration evidence and must remain until release.
