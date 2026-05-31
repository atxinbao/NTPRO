# RADP-010 Evidence - Inventory Rust Adapter Gaps For Deribit dYdX Hyperliquid

Date: 2026-05-31
Executor: Codex
Task ID: RADP-010
Risk: medium

## Summary

Created the Deribit, dYdX, and Hyperliquid Rust adapter gap inventory for the
R5 adapter parity sequence. The inventory records current Rust surfaces,
fixture/test coverage, support classifications, and release-gate gaps for
RADP-011 and RADP-012.

The task is documentation and control-state only. It does not change adapter
runtime behavior, exchange protocol handling, order routing, credential
handling, public APIs, Python/PyO3 bindings, Cython surfaces, or Cargo feature
behavior.

## Files Changed

- `docs/rust-cutover/inventory/deribit_dydx_hyperliquid_adapter_gaps.md`
- `docs/rust-cutover/evidence/RADP-010.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-010.json`

## Commands Run

Context and inventory:

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RADP-010.md
sed -n '1,220p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,220p' docs/rust-cutover/CONTRACT.md
sed -n '1,220p' docs/rust-cutover/DEFINITION_OF_DONE.md
sed -n '1,220p' docs/rust-cutover/AGENT_ROLES.md
python3 scripts/ai/lease.py claim RADP-010 --branch ai/RADP-010-inventory-rust-adapter-gaps-for-deribit-dydx-hyperliquid --agent-id Codex --path docs/rust-cutover/tasks/RADP-010.md --path docs/rust-cutover/evidence/RADP-010.md --path .agentflow/state/task_status.json --path .agentflow/leases/RADP-010.json
find crates/adapters/deribit crates/adapters/dydx crates/adapters/hyperliquid -maxdepth 3 -type f | sort
rg -n "unsupported|Unsupported|todo!|unimplemented!|panic!|bail!|anyhow!|return Err|subscribe_|request_|AccountType|OrderType|TimeInForce|InstrumentClass|ProductType|Python|pyo3|feature = \"python\"|cfg\\(feature = \"python\"\\)" crates/adapters/deribit/src crates/adapters/dydx/src crates/adapters/hyperliquid/src crates/adapters/deribit/README.md crates/adapters/dydx/README.md crates/adapters/hyperliquid/README.md
```

Targeted source reads:

```bash
sed -n '1,220p' crates/adapters/deribit/Cargo.toml
sed -n '1,180p' crates/adapters/deribit/src/lib.rs
sed -n '1,180p' crates/adapters/deribit/src/common/enums.rs
sed -n '150,210p' crates/adapters/deribit/src/common/parse.rs
sed -n '520,620p' crates/adapters/deribit/src/data.rs
sed -n '720,840p' crates/adapters/deribit/src/data.rs
sed -n '960,1085p' crates/adapters/deribit/src/data.rs
sed -n '1140,1225p' crates/adapters/deribit/src/data.rs
sed -n '1,260p' crates/adapters/deribit/src/execution.rs
sed -n '1,220p' crates/adapters/dydx/Cargo.toml
sed -n '1,180p' crates/adapters/dydx/src/lib.rs
sed -n '160,280p' crates/adapters/dydx/src/common/enums.rs
sed -n '1040,1125p' crates/adapters/dydx/src/http/client.rs
sed -n '1200,1325p' crates/adapters/dydx/src/execution/mod.rs
sed -n '1380,1570p' crates/adapters/dydx/src/execution/mod.rs
sed -n '1,220p' crates/adapters/hyperliquid/Cargo.toml
sed -n '1,180p' crates/adapters/hyperliquid/src/lib.rs
sed -n '180,520p' crates/adapters/hyperliquid/src/data.rs
sed -n '360,650p' crates/adapters/hyperliquid/src/common/parse.rs
sed -n '2060,2140p' crates/adapters/hyperliquid/src/execution.rs
```

Required and final validation:

```bash
scripts/ai/verify_fast.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-010.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- `scripts/ai/verify_fast.sh`: passed; output ended with `== verify_fast complete ==`.
- Final JSON validation for task state and lease: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed with `agentflow role protocol validation passed`.
- `git diff --check`: passed.

## Tests Added Or Updated

No tests were added in RADP-010. This task is an inventory-only adapter parity
task. RADP-011 is the follow-up fixture task for the gaps recorded here.

## Behavior Impact

No runtime behavior changed. No parser, data-client, execution-client,
WebSocket, gRPC, HTTP, transaction-broadcast, signing, outcome-settlement,
credential, order routing, public API, Python API, PyO3 binding, Cython
surface, Cargo feature, or persistence behavior changed.

The practical impact is release-gate clarity: Deribit, dYdX, and Hyperliquid
adapter parity gaps are now explicit follow-up inputs for fixture coverage and
closure decisions.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR documents adapter gaps without
changing runtime behavior or public APIs.

## Gate Status

RADP-010 is medium risk. It creates adapter inventory evidence only; it does not
change adapter runtime behavior or trading semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the inventory file, this evidence file, and the RADP-010 task state/lease
updates. No runtime, persisted data, adapter protocol, schema, Python, PyO3,
Cython, or public API rollback is required.
