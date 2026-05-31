# RADP-005 Evidence - Add Rust Adapter Fixtures For Bybit OKX Kraken

Date: 2026-05-31
Executor: Codex
Task ID: RADP-005
Risk: medium

## Summary

Added Rust fixture coverage manifests for the Bybit, OKX, and Kraken adapters,
plus one Rust manifest test for each adapter. The manifests bind the existing
fixture corpus to the RADP adapter parity gate and record scoped blockers from
the RADP-004 inventory for RADP-006 closure.

This task does not add new exchange payloads and does not change parser,
data-client, or execution-client behavior. It makes the current fixture and
blocker coverage machine-checkable.

## Files Changed

- `crates/adapters/bybit/test_data/rust_fixture_manifest.json`
- `crates/adapters/bybit/tests/fixture_manifest.rs`
- `crates/adapters/okx/test_data/rust_fixture_manifest.json`
- `crates/adapters/okx/tests/fixture_manifest.rs`
- `crates/adapters/kraken/test_data/rust_fixture_manifest.json`
- `crates/adapters/kraken/tests/fixture_manifest.rs`
- `docs/rust-cutover/evidence/RADP-005.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-005.json`

## Commands Run

Task setup and context:

```bash
git status --short --branch
sed -n '1,220p' docs/rust-cutover/tasks/RADP-005.md
sed -n '1,260p' docs/rust-cutover/evidence/RADP-002.md
sed -n '1,260p' crates/adapters/binance/test_data/rust_fixture_manifest.json
sed -n '1,260p' crates/adapters/binance/tests/fixture_manifest.rs
find crates/adapters/bybit/test_data -maxdepth 1 -type f
find crates/adapters/okx/test_data -maxdepth 1 -type f
find crates/adapters/kraken/test_data -maxdepth 1 -type f
```

Targeted validation:

```bash
python3 -m json.tool crates/adapters/bybit/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool crates/adapters/okx/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool crates/adapters/kraken/test_data/rust_fixture_manifest.json >/dev/null
cargo fmt --check
cargo fmt
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-bybit --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-okx --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-kraken --test fixture_manifest
```

Required and final validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-005.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- Fixture manifest JSON validation for Bybit, OKX, and Kraken: passed.
- First `cargo fmt --check`: failed because the new manifest tests needed
  standard rustfmt wrapping.
- `cargo fmt`: passed.
- Rust 1.95 `cargo test -p nautilus-bybit --test fixture_manifest`: passed,
  1 test.
- Rust 1.95 `cargo test -p nautilus-okx --test fixture_manifest`: passed,
  1 test.
- Rust 1.95 `cargo test -p nautilus-kraken --test fixture_manifest`: passed,
  1 test.
- Rust 1.95 `scripts/ai/verify_full.sh`: passed and ended with
  `== verify_full complete ==`.
- Final JSON validation for task state and lease: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.

## Tests Added Or Updated

Added:

- `crates/adapters/bybit/tests/fixture_manifest.rs`
- `crates/adapters/okx/tests/fixture_manifest.rs`
- `crates/adapters/kraken/tests/fixture_manifest.rs`

Each test verifies:

- the adapter fixture manifest is valid JSON;
- the task ID is `RADP-005`;
- the inventory task is `RADP-004`;
- required parser/lifecycle surfaces are classified;
- every fixture path listed in the manifest exists under the adapter's
  `test_data/` directory;
- every primary test/parser file listed in the manifest exists;
- every scoped blocker links back to the RADP-004 inventory gap IDs and is
  assigned to RADP-006 for closure.

## Behavior Impact

No runtime behavior changed. No parser, data-client, execution-client, exchange
protocol, credential, public API, Python API, PyO3 binding, Cython surface,
Cargo feature, or persistence behavior changed.

The practical impact is release-gate evidence: existing Bybit, OKX, and Kraken
fixtures are now grouped into machine-checkable Rust adapter parity manifests.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR adds fixture coverage metadata and
manifest tests only.

## Gate Status

RADP-005 is medium risk. It improves adapter fixture evidence without changing
adapter runtime behavior or trading semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the three adapter fixture manifests, three manifest tests, this evidence
file, and the RADP-005 task state/lease updates. No runtime, persisted data,
adapter protocol, schema, Python, PyO3, Cython, or public API rollback is
required.
