# RADP-008 Evidence - Add Rust Adapter Fixtures For Coinbase BitMEX

Date: 2026-05-31
Executor: Codex
Task ID: RADP-008
Risk: medium

## Summary

Added Rust fixture coverage manifests for the Coinbase and BitMEX adapters,
plus one Rust manifest test for each adapter. The manifests bind the existing
fixture corpus to the RADP adapter parity gate and record scoped blockers from
the RADP-007 inventory for RADP-009 closure.

This task does not add new exchange payloads and does not change parser,
data-client, execution-client, broadcaster, dead man's switch, or WebSocket
behavior. It makes the current Coinbase and BitMEX fixture and blocker coverage
machine-checkable.

## Files Changed

- `crates/adapters/coinbase/test_data/rust_fixture_manifest.json`
- `crates/adapters/coinbase/tests/fixture_manifest.rs`
- `crates/adapters/bitmex/test_data/rust_fixture_manifest.json`
- `crates/adapters/bitmex/tests/fixture_manifest.rs`
- `docs/rust-cutover/evidence/RADP-008.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-008.json`

## Commands Run

Task setup and context:

```bash
git status --short --branch
sed -n '1,220p' docs/rust-cutover/tasks/RADP-008.md
sed -n '1,220p' docs/rust-cutover/AGENT_ROLES.md
sed -n '1,220p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,260p' docs/rust-cutover/inventory/coinbase_bitmex_adapter_gaps.md
rg --files crates/adapters/coinbase | sort
rg --files crates/adapters/bitmex | sort
```

Targeted validation:

```bash
python3 -m json.tool crates/adapters/coinbase/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool crates/adapters/bitmex/test_data/rust_fixture_manifest.json >/dev/null
cargo fmt --check
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-coinbase --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" cargo test -p nautilus-bitmex --test fixture_manifest
```

Required and final validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-008.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- Fixture manifest JSON validation for Coinbase and BitMEX: passed.
- `cargo fmt --check`: passed.
- First Rust 1.95 `cargo test -p nautilus-coinbase --test fixture_manifest`:
  failed because the initial manifest reused fixture paths across groups.
- Coinbase and BitMEX manifests were adjusted so each fixture path is classified
  once.
- Rust 1.95 `cargo test -p nautilus-coinbase --test fixture_manifest`: passed,
  1 test.
- Rust 1.95 `cargo test -p nautilus-bitmex --test fixture_manifest`: passed,
  1 test.
- Rust 1.95 `scripts/ai/verify_full.sh`: passed and ended with
  `== verify_full complete ==`.

## Tests Added Or Updated

Added:

- `crates/adapters/coinbase/tests/fixture_manifest.rs`
- `crates/adapters/bitmex/tests/fixture_manifest.rs`

Each test verifies:

- the adapter fixture manifest is valid JSON;
- the task ID is `RADP-008`;
- the inventory task is `RADP-007`;
- required parser, lifecycle, and operational surfaces are classified;
- every fixture path listed in the manifest exists under the adapter's
  `test_data/` directory;
- every primary test/parser file listed in the manifest exists;
- every scoped blocker links back to the RADP-007 inventory gap IDs and is
  assigned to RADP-009 for closure.

## Behavior Impact

No runtime behavior changed. No parser, data-client, execution-client,
broadcaster, dead man's switch, exchange protocol, credential, public API,
Python API, PyO3 binding, Cython surface, Cargo feature, or persistence behavior
changed.

The practical impact is release-gate evidence: existing Coinbase and BitMEX
fixtures are now grouped into machine-checkable Rust adapter parity manifests.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR adds fixture coverage metadata and
manifest tests only.

## Gate Status

RADP-008 is medium risk. It improves adapter fixture evidence without changing
adapter runtime behavior or trading semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the two adapter fixture manifests, two manifest tests, this evidence
file, and the RADP-008 task state/lease updates. No runtime, persisted data,
adapter protocol, schema, Python, PyO3, Cython, or public API rollback is
required.
