# RADP-006 Evidence - Close Rust Adapter Gaps For Bybit OKX Kraken

Date: 2026-05-31
Executor: Codex
Task ID: RADP-006
Risk: medium

## Summary

Closed the RADP-004 Bybit, OKX, and Kraken adapter gap inventory by recording
explicit Rust-first support decisions for every gap. RADP-005 already added
machine-checkable fixture manifests for the three adapters, so this task does
not need runtime code changes to satisfy the closure contract.

The closure result is:

- compact fixture-backed manifests now exist for Bybit, OKX, and Kraken;
- product and order-type limitations are explicitly scoped as supported with
  constraints;
- optional Python/PyO3 adapter surfaces remain deferred to the removal gate;
- no parser, data-client, execution-client, exchange protocol, credential, or
  public API behavior changed.

## Files Changed

- `docs/rust-cutover/evidence/RADP-006.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-006.json`

## Closure Decisions

| Gap | Closure | Evidence |
| --- | --- | --- |
| BYB-ADP-001 | Closed by RADP-005 manifest coverage. | `crates/adapters/bybit/test_data/rust_fixture_manifest.json` classifies the Bybit HTTP, WebSocket, and execution lifecycle surfaces. |
| BYB-ADP-002 | Scoped as supported with constraints. | Bybit keeps separate public data WebSockets per product type and shared private/trade execution clients with product routing; this is an explicit runtime shape, not an unresolved release blocker. |
| BYB-ADP-003 | Scoped as supported with constraints. | Only L2 MBP depths `1`, `50`, `200`, and `500` are supported; unsupported book modes remain out of scope. |
| BYB-ADP-004 | Scoped as supported with constraints. | Product-specific unsupported funding, mark/index, and bar combinations remain explicit support limits. |
| BYB-ADP-005 | Scoped as supported with constraints. | Unsupported order and demo WebSocket Trade API cases remain explicit support limits instead of silent parity claims. |
| BYB-ADP-006 | Deferred to removal gate. | Optional Python/PyO3 surfaces are not authorized for deletion by RADP-006. |
| OKX-ADP-001 | Closed by RADP-005 manifest coverage. | `crates/adapters/okx/test_data/rust_fixture_manifest.json` classifies OKX HTTP, WebSocket, execution, option, spread, and event-adjacent surfaces. |
| OKX-ADP-002 | Scoped as supported with constraints. | Options and events intentionally do not share every regular order/algo path. |
| OKX-ADP-003 | Scoped as supported with constraints. | Spread orders remain separate from the normal order path and keep documented limits for lists, conditionals, FOK, and modify requests. |
| OKX-ADP-004 | Scoped as supported with constraints. | L2 MBP-only order-book channel/depth selection remains the supported Rust behavior. |
| OKX-ADP-005 | Scoped as supported with constraints. | Option loading requires configured instrument families; zero-config option parity is not claimed. |
| OKX-ADP-006 | Deferred to removal gate. | Optional Python/PyO3 surfaces are not authorized for deletion by RADP-006. |
| KRK-ADP-001 | Closed by RADP-005 manifest coverage. | `crates/adapters/kraken/test_data/rust_fixture_manifest.json` classifies Kraken Spot, Futures, WebSocket, and execution lifecycle surfaces. |
| KRK-ADP-002 | Scoped as supported with constraints. | Kraken Spot demo remains unsupported by config validation. |
| KRK-ADP-003 | Scoped as supported with constraints. | Kraken Spot L3 remains credentialed and limited to fixed supported depths. |
| KRK-ADP-004 | Scoped as supported with constraints. | Kraken Spot and Futures keep different data capability surfaces. |
| KRK-ADP-005 | Scoped as supported with constraints. | Unsupported WebSocket order instructions remain REST-routed or deferred rather than claimed as WS parity. |
| KRK-ADP-006 | Deferred to removal gate. | Optional Python/PyO3 surfaces are not authorized for deletion by RADP-006. |

## Commands Run

Context and status:

```bash
git status --short --branch
sed -n '1,260p' docs/rust-cutover/tasks/RADP-006.md
sed -n '1,260p' docs/rust-cutover/inventory/bybit_okx_kraken_adapter_gaps.md
sed -n '1,260p' docs/rust-cutover/evidence/RADP-005.md
sed -n '1,160p' .agentflow/leases/RADP-006.json
sed -n '84,120p' .agentflow/state/task_status.json
```

Required and final validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-006.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- Rust 1.95 `scripts/ai/verify_full.sh`: passed; output ended with
  `== verify_full complete ==`.
- Final JSON validation for task state and lease: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed with
  `agentflow role protocol validation passed`.
- `git diff --check`: passed.

## Tests Added Or Updated

No tests were added in RADP-006. The executable fixture checks were added in
RADP-005:

- `crates/adapters/bybit/tests/fixture_manifest.rs`
- `crates/adapters/okx/tests/fixture_manifest.rs`
- `crates/adapters/kraken/tests/fixture_manifest.rs`

RADP-006 records the closure decisions for those manifests and the RADP-004 gap
inventory.

## Behavior Impact

No runtime behavior changed. No parser, data-client, execution-client, exchange
protocol, credential handling, order routing, public API, Python API, PyO3
binding, Cython surface, Cargo feature, or persistence behavior changed.

The practical impact is release-gate clarity: Bybit, OKX, and Kraken adapter
parity is now represented as supported-with-constraints or deferred removal-gate
scope instead of unresolved inventory gaps.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR documents adapter parity closure
without changing runtime behavior or public APIs.

## Rollback Plan

Revert this evidence file and the RADP-006 task state/lease updates. No runtime,
persisted data, adapter protocol, schema, Python, PyO3, Cython, or public API
rollback is required.
