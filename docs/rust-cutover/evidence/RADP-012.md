# RADP-012 Evidence - Close Rust Adapter Gaps For Deribit dYdX Hyperliquid

Date: 2026-05-31
Executor: Codex
Task ID: RADP-012
Risk: medium

## Summary

Closed the RADP-010 Deribit, dYdX, and Hyperliquid adapter gap inventory by
recording explicit Rust-first closure decisions in each adapter fixture
manifest. RADP-011 already added machine-checkable manifests and fixture tests,
so this task keeps runtime behavior unchanged and turns the remaining open
adapter gaps into closed, scoped, or deferred release-gate decisions.

The closure result is:

- compact fixture-backed manifests now have `gap_closure` entries for all
  Deribit, dYdX, and Hyperliquid gaps;
- manifest tests now reject `open` gap status for RADP-012-owned blockers;
- product, book, market-data, execution, operational, and outcome-settlement
  limits are explicitly scoped as supported with constraints;
- optional Python/PyO3 adapter surfaces remain deferred to the removal gate;
- no parser, data-client, execution-client, WebSocket, gRPC, HTTP, signing,
  account, exchange protocol, credential, or public API behavior changed.

## Files Changed

- `crates/adapters/deribit/test_data/rust_fixture_manifest.json`
- `crates/adapters/deribit/tests/fixture_manifest.rs`
- `crates/adapters/dydx/test_data/rust_fixture_manifest.json`
- `crates/adapters/dydx/tests/fixture_manifest.rs`
- `crates/adapters/hyperliquid/test_data/rust_fixture_manifest.json`
- `crates/adapters/hyperliquid/tests/fixture_manifest.rs`
- `docs/rust-cutover/evidence/RADP-012.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-012.json`

## Closure Decisions

| Gap | Closure | Evidence |
| --- | --- | --- |
| DER-ADP-001 | Closed by RADP-011 manifest coverage. | `crates/adapters/deribit/test_data/rust_fixture_manifest.json` classifies Deribit HTTP, WebSocket, execution, and RPC lifecycle fixture groups. |
| DER-ADP-002 | Scoped as supported with constraints. | Deribit product coverage includes futures, options, spot, and combos, while zero-config data bootstrap defaults to futures. |
| DER-ADP-003 | Scoped as supported with constraints. | Deribit order books remain L2 MBP with explicit raw/grouped depth behavior. |
| DER-ADP-004 | Scoped as supported with constraints. | Deribit market data remains ticker/perpetual-channel specific with supported bar resolutions and volatility-index custom data. |
| DER-ADP-005 | Scoped as supported with constraints. | Deribit execution remains authenticated margin execution with the current order and TIF matrix. |
| DER-ADP-006 | Deferred to removal gate. | Optional Deribit Python/PyO3 surfaces are not authorized for deletion by RADP-012. |
| DYDX-ADP-001 | Closed by RADP-011 manifest coverage. | `crates/adapters/dydx/test_data/rust_fixture_manifest.json` classifies dYdX HTTP, WebSocket, account/execution, and block gRPC fixture groups. |
| DYDX-ADP-002 | Scoped as supported with constraints. | dYdX Rust instrument support remains v4 perpetual-market only. |
| DYDX-ADP-003 | Scoped as supported with constraints. | dYdX market data remains bound to v4 Indexer channel semantics and candle resolutions. |
| DYDX-ADP-004 | Scoped as supported with constraints. | dYdX execution keeps wallet/subaccount, block-height, broadcaster, gRPC, order, and TIF limits. |
| DYDX-ADP-005 | Scoped as supported with constraints. | dYdX block monitoring, broadcaster, subaccount WebSocket, hashed client IDs, and order correlation are release-gate visible through the operational manifest group. |
| DYDX-ADP-006 | Deferred to removal gate. | Optional dYdX Python/PyO3 surfaces are not authorized for deletion by RADP-012. |
| HYP-ADP-001 | Closed by RADP-011 manifest coverage. | `crates/adapters/hyperliquid/test_data/rust_fixture_manifest.json` classifies Hyperliquid HTTP, WebSocket, account/funding, and outcome operational fixture groups. |
| HYP-ADP-002 | Scoped as supported with constraints. | Hyperliquid product support remains perps, spot, and best-effort HIP-4 outcomes. |
| HYP-ADP-003 | Scoped as supported with constraints. | Hyperliquid market data remains L2 MBP plus all-mids and open-interest custom streams. |
| HYP-ADP-004 | Scoped as supported with constraints. | Hyperliquid execution keeps IOC-derived market orders, current TIF restrictions, CLOID handling, and HIP-4 outcome limits. |
| HYP-ADP-005 | Scoped as supported with constraints. | Hyperliquid WebSocket post execution, CLOID cache, address resolution, clearinghouse reconciliation, and optional outcome polling are release-gate visible through the operational manifest group. |
| HYP-ADP-006 | Deferred to removal gate. | Optional Hyperliquid Python/PyO3 surfaces are not authorized for deletion by RADP-012. |

## Commands Run

Context and status:

```bash
git status --short --branch
sed -n '1,220p' docs/rust-cutover/tasks/RADP-012.md
sed -n '1,260p' docs/rust-cutover/inventory/deribit_dydx_hyperliquid_adapter_gaps.md
sed -n '1,220p' docs/rust-cutover/evidence/RADP-011.md
sed -n '1,180p' .agentflow/leases/RADP-012.json
python3 -m json.tool .agentflow/state/task_status.json
```

Targeted and required validation:

```bash
python3 -m json.tool crates/adapters/deribit/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool crates/adapters/dydx/test_data/rust_fixture_manifest.json >/dev/null
python3 -m json.tool crates/adapters/hyperliquid/test_data/rust_fixture_manifest.json >/dev/null
cargo fmt --check
cargo fmt
cargo fmt --check
cargo test -p nautilus-deribit --test fixture_manifest
cargo test -p nautilus-dydx --test fixture_manifest
cargo test -p nautilus-hyperliquid --test fixture_manifest
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-012.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- Fixture manifest JSON validation: passed.
- First `cargo fmt --check`: failed only because rustfmt wanted standard line
  wrapping in the updated manifest tests.
- `cargo fmt`: passed.
- Final `cargo fmt --check`: passed.
- Rust 1.95 `cargo test -p nautilus-deribit --test fixture_manifest`: passed,
  1 test.
- Rust 1.95 `cargo test -p nautilus-dydx --test fixture_manifest`: passed,
  1 test.
- Rust 1.95 `cargo test -p nautilus-hyperliquid --test fixture_manifest`:
  passed, 1 test.
- Rust 1.95 `scripts/ai/verify_full.sh`: passed and ended with
  `== verify_full complete ==`.
- Final JSON validation for task state and lease: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed with
  `agentflow role protocol validation passed`.
- `git diff --check`: passed.

## Tests Added Or Updated

Updated:

- `crates/adapters/deribit/tests/fixture_manifest.rs`
- `crates/adapters/dydx/tests/fixture_manifest.rs`
- `crates/adapters/hyperliquid/tests/fixture_manifest.rs`

The tests now verify:

- every RADP-012-owned blocker has a non-open status;
- every blocker has a RADP-012 resolution;
- every `gap_closure` entry records a non-open status, review task, decision,
  and evidence references;
- all RADP-010 gap IDs for Deribit, dYdX, and Hyperliquid are represented.

## Behavior Impact

No runtime behavior changed. No parser, data-client, execution-client,
WebSocket, gRPC, HTTP, signing, account handling, order routing, exchange
protocol, credential handling, public API, Python API, PyO3 binding, Cython
surface, Cargo feature behavior, or persistence behavior changed.

The practical impact is release-gate clarity: Deribit, dYdX, and Hyperliquid
adapter parity is now represented as supported-with-constraints or deferred
removal-gate scope instead of unresolved inventory gaps.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR documents adapter parity closure
without changing runtime behavior or public APIs.

## Gate Status

RADP-012 is medium risk. It closes adapter parity gaps by scope decision and
manifest validation only; it does not change adapter runtime behavior or
trading semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the Deribit, dYdX, and Hyperliquid manifest closure entries, manifest
test assertions, this evidence file, and the RADP-012 task state/lease updates.
No runtime, persisted data, adapter protocol, schema, Python, PyO3, Cython, or
public API rollback is required.
