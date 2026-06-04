# Post-release Developer Guide Python Surface Cleanup Evidence

- Date: 2026-06-05
- Executor: Codex
- Local task name: `POST-RELEASE-DEVELOPER-GUIDE-PYTHON-SURFACE-CLEANUP`
- Formal task file: not present in the Shrimp queue; this is a scoped
  post-release public surface cleanup.

## Goal

Clean developer-guide pages where Python, PyO3, Cython, pytest, virtualenv, or
legacy adapter-layer wording could be read as a current NTPRO product
development path.

## Scope

Changed:

- `docs/developer_guide/adapters.md`
- `docs/developer_guide/environment_setup.md`
- `docs/developer_guide/ffi.md`
- `docs/developer_guide/python.md`
- `docs/developer_guide/spec_data_testing.md`
- `docs/developer_guide/spec_exec_testing.md`
- `docs/developer_guide/testing.md`

Not changed:

- Rust source code;
- adapter behavior;
- trading semantics;
- Cargo features, crate names, or workspace metadata;
- CLI, dashboard, or control API implementation;
- Python, PyO3, Cython, `build.py`, or package files;
- release tags or GitHub Releases.

## Changes

- Reframed adapter development as Rust-only:
  - removed Python/PyO3 adapter layer from the active implementation sequence;
  - changed data/execution client steps from Python wiring to Rust wiring;
  - changed adapter evidence from Python integration tests to Rust tests,
    fixtures, mocks, or spec cards.
- Reframed environment setup as Rust/Cargo-first:
  - removed `PYO3_PYTHON`, `PYTHONHOME`, `uv sync`, and `make build-debug`
    from the quick setup product path;
  - marked Python/PyO3 environment variables and uv dependency management as
    legacy upstream context.
- Reframed testing as Rust/Cargo-first:
  - changed pytest and PyO3 test sections to legacy upstream context;
  - removed `python` from the current Rust nextest feature example;
  - marked PyO3/Python/Cython data-type test layers as legacy context.
- Reframed spec data/execution testing:
  - made Rust `nautilus_testkit::testers` the current NTPRO evidence path;
  - marked Python node setup and Python config blocks as legacy upstream
    examples.
- Reframed `python.md` and `ffi.md`:
  - marked Python/PyO3/Cython conventions as legacy upstream notes;
  - clarified that new NTPRO product work must not add Python/PyO3/Cython
    product paths.

## Keyword Classification

The broad keyword count is intentionally not zero because retained historical
material is allowed when clearly labeled.

| File | Before | After | Classification |
|------|--------|-------|----------------|
| `docs/developer_guide/adapters.md` | 18 | 13 | Current adapter steps no longer require Python/PyO3; retained mentions are legacy or negative guidance. |
| `docs/developer_guide/testing.md` | 20 | 22 | Retained testing sections now explicitly say legacy upstream or not product evidence. |
| `docs/developer_guide/environment_setup.md` | 13 | 12 | Current setup path is Rust/Cargo; Python/PyO3 variables are legacy context. |
| `docs/developer_guide/spec_data_testing.md` | 28 | 28 | Python config examples retained but labeled legacy upstream. |
| `docs/developer_guide/spec_exec_testing.md` | 42 | 42 | Python config examples retained but labeled legacy upstream. |
| `docs/developer_guide/python.md` | 5 | 9 | Added explicit unsupported/legacy framing. |
| `docs/developer_guide/ffi.md` | 8 | 11 | Added explicit unsupported/legacy framing. |

## Validation

| Command | Result | Notes |
|---------|--------|-------|
| targeted current-path `rg` | passed | No matches for phrases such as `new Rust-backed PyO3 adapters`, `Expose Rust functionality via PyO3`, `Required for Rust/PyO3`, `Run tests with pytest`, or active Python adapter-client steps. Command exited 1 because `rg` found no current-path wording. |
| broad legacy keyword `rg` | completed | 137 retained hits for Python/PyO3/Cython-related terms, classified as legacy upstream, unsupported, or negative guidance. |
| `git diff --check` | passed | No whitespace errors before evidence creation. |
| `scripts/ai/check_rust_only_runtime.sh` | passed | Reported `rust-only-runtime: ok`. |
| `scripts/ai/check_cython_removed.sh` | passed | Reported `cython-removed: ok`. |
| `scripts/ai/verify_fast.sh` | passed | Used cargo/rustc 1.95.0; workspace cargo check and clippy skipped by fast-mode defaults. |
| `bash -lc 'source scripts/ai/toolchain_env.sh && cargo check -p nautilus-cli'` | passed | Finished `dev` profile in 0.66s. |

## Behavior Impact

No runtime behavior changed. This is a documentation-only cleanup.

## Public API Impact

No public Rust API changed. No exported types, functions, modules, Cargo
features, crate names, binary names, or adapter APIs changed.

## Migration Note Status

No new migration note is required. This PR reinforces the existing Rust-only
post-release posture by marking developer-guide Python, PyO3, and Cython
material as unsupported legacy context.

## Rollback Plan

Revert this PR to restore the prior developer-guide wording. No runtime,
dependency, API, data, or release rollback is required.
