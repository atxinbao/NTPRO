# RADP-009 Evidence - Close Rust Adapter Gaps For Coinbase BitMEX

Date: 2026-05-31
Executor: Codex
Task ID: RADP-009
Risk: medium

## Summary

Closed the RADP-007 Coinbase and BitMEX adapter gap inventory by recording
explicit Rust-first support decisions for every gap. RADP-008 already added
machine-checkable fixture manifests for both adapters, so this task does not
need runtime code changes to satisfy the closure contract.

The closure result is:

- compact fixture-backed manifests now exist for Coinbase and BitMEX;
- product, book, bar, order, account-type, and operational limitations are
  explicitly scoped as supported with constraints;
- optional Python/PyO3 adapter surfaces remain deferred to the removal gate;
- no parser, data-client, execution-client, exchange protocol, credential,
  WebSocket, broadcaster, dead man's switch, or public API behavior changed.

## Files Changed

- `docs/rust-cutover/evidence/RADP-009.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-009.json`

## Closure Decisions

| Gap | Closure | Evidence |
| --- | --- | --- |
| CB-ADP-001 | Closed by RADP-008 manifest coverage. | `crates/adapters/coinbase/test_data/rust_fixture_manifest.json` classifies Coinbase HTTP, WebSocket, execution, user-channel, and derivatives polling surfaces. |
| CB-ADP-002 | Scoped as supported with constraints. | Coinbase Rust support remains limited to crypto spot and CFM derivatives; generic Advanced Trade product parity is not claimed. |
| CB-ADP-003 | Scoped as supported with constraints. | Mark prices remain explicitly unsupported; index and funding data remain REST-polled derivative surfaces. |
| CB-ADP-004 | Scoped as supported with constraints. | Coinbase order-book streaming remains scoped to L2 MBP deltas and does not claim other book modes. |
| CB-ADP-005 | Scoped as supported with constraints. | Coinbase execution keeps the existing market, limit, and stop-limit matrix with unsupported TIF/order-type combinations rejected. |
| CB-ADP-006 | Scoped as supported with constraints. | Coinbase spot and CFM execution remain selected by one account-type flag per client instance. |
| CB-ADP-007 | Deferred to removal gate. | Optional Coinbase Python/PyO3 surfaces are not authorized for deletion by RADP-009. |
| BMX-ADP-001 | Closed by RADP-008 manifest coverage. | `crates/adapters/bitmex/test_data/rust_fixture_manifest.json` classifies BitMEX HTTP, WebSocket, execution, private lifecycle, and operational lifecycle surfaces. |
| BMX-ADP-002 | Scoped as supported with constraints. | BitMEX support remains limited to spot, perpetuals, futures, prediction markets, and index instruments; unsupported venue instrument classes are not claimed. |
| BMX-ADP-003 | Scoped as supported with constraints. | BitMEX book support remains L2/depth10 with exchange-specific depth behavior. |
| BMX-ADP-004 | Scoped as supported with constraints. | BitMEX bar support remains limited to external last-price bars for selected intervals. |
| BMX-ADP-005 | Scoped as supported with constraints. | BitMEX execution remains authenticated margin/netting behavior with explicit order constraints. |
| BMX-ADP-006 | Scoped as supported with constraints. | BitMEX broadcaster and dead man's switch surfaces are classified by RADP-008 operational lifecycle evidence; live exchange side effects are not claimed without a dedicated sandbox/mock gate. |
| BMX-ADP-007 | Deferred to removal gate. | Optional BitMEX Python/PyO3 surfaces are not authorized for deletion by RADP-009. |

## Commands Run

Context and status:

```bash
git status --short --branch
sed -n '1,220p' docs/rust-cutover/tasks/RADP-009.md
sed -n '1,260p' docs/rust-cutover/inventory/coinbase_bitmex_adapter_gaps.md
sed -n '1,220p' docs/rust-cutover/evidence/RADP-008.md
sed -n '1,180p' .agentflow/leases/RADP-009.json
python3 -m json.tool .agentflow/state/task_status.json
```

Required and final validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-009.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- Rust 1.95 `scripts/ai/verify_full.sh`: passed; output ended with `== verify_full complete ==`.
- Final JSON validation for task state and lease: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed with `agentflow role protocol validation passed`.
- `git diff --check`: passed.

## Tests Added Or Updated

No tests were added in RADP-009. The executable fixture checks were added in
RADP-008:

- `crates/adapters/coinbase/tests/fixture_manifest.rs`
- `crates/adapters/bitmex/tests/fixture_manifest.rs`

RADP-009 records the closure decisions for those manifests and the RADP-007 gap
inventory.

## Behavior Impact

No runtime behavior changed. No parser, data-client, execution-client,
broadcaster, dead man's switch, exchange protocol, credential handling, order
routing, public API, Python API, PyO3 binding, Cython surface, Cargo feature, or
persistence behavior changed.

The practical impact is release-gate clarity: Coinbase and BitMEX adapter parity
is now represented as supported-with-constraints or deferred removal-gate scope
instead of unresolved inventory gaps.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR documents adapter parity closure
without changing runtime behavior or public APIs.

## Gate Status

RADP-009 is medium risk. It closes adapter parity gaps by scope decision and
manifest evidence only; it does not change adapter runtime behavior or trading
semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert this evidence file and the RADP-009 task state/lease updates. No runtime,
persisted data, adapter protocol, schema, Python, PyO3, Cython, or public API
rollback is required.
