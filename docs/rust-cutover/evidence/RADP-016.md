# RADP-016 Evidence - Inventory Rust Adapter Gaps For Interactive Brokers

Date: 2026-05-31
Executor: Codex
Task ID: RADP-016
Risk: medium

## Summary

Created the Interactive Brokers Rust adapter gap inventory for the R5 adapter
parity sequence. The inventory records current Rust surfaces, fixture/test
coverage, support classifications, and release-gate gaps for RADP-017 and
RADP-018.

The task is documentation and control-state only. It does not change adapter
runtime behavior, market-data handling, historical-data handling, order
execution, account handling, gateway behavior, credential handling, public APIs,
Python/PyO3 bindings, Cython surfaces, or Cargo feature behavior.

## Files Changed

- `docs/rust-cutover/inventory/interactive_brokers_adapter_gaps.md`
- `docs/rust-cutover/evidence/RADP-016.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-016.json`

## Commands Run

Context and inventory:

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RADP-016.md
sed -n '1,260p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,220p' docs/rust-cutover/CONTRACT.md
sed -n '1,220p' docs/rust-cutover/DEFINITION_OF_DONE.md
sed -n '1,260p' docs/rust-cutover/AGENT_ROLES.md
sed -n '1,260p' docs/rust-cutover/inventory/databento_tardis_adapter_gaps.md
rg --files crates/adapters/interactive_brokers docs/rust-cutover/inventory docs/rust-cutover/evidence
sed -n '1,240p' crates/adapters/interactive_brokers/README.md
sed -n '1,260p' crates/adapters/interactive_brokers/Cargo.toml
sed -n '1,260p' crates/adapters/interactive_brokers/src/lib.rs
find crates/adapters/interactive_brokers/tests crates/adapters/interactive_brokers/examples -type f | sort
rg -n '#\[(rstest|test|tokio::test)' crates/adapters/interactive_brokers/src crates/adapters/interactive_brokers/tests | wc -l
rg -n 'fn (connect|disconnect|subscribe_|unsubscribe_|request_|new|submit_|modify_|cancel_|generate_|parse_|handle_|resolve_)' crates/adapters/interactive_brokers/src/data crates/adapters/interactive_brokers/src/historical crates/adapters/interactive_brokers/src/providers crates/adapters/interactive_brokers/src/execution
rg -n 'bail!|Unsupported|unsupported|does not support|not supported|TODO|todo|warn!|error!' crates/adapters/interactive_brokers/src/data crates/adapters/interactive_brokers/src/historical crates/adapters/interactive_brokers/src/providers crates/adapters/interactive_brokers/src/execution
rg -n 'pyclass|pyo3|feature = "python"|cfg\(feature = "python"\)|extension-module|python' crates/adapters/interactive_brokers/src crates/adapters/interactive_brokers/Cargo.toml
```

Targeted source reads:

```bash
sed -n '1,260p' crates/adapters/interactive_brokers/src/config.rs
sed -n '1,280p' crates/adapters/interactive_brokers/src/data/core.rs
sed -n '1,260p' crates/adapters/interactive_brokers/src/data/parse.rs
sed -n '1,240p' crates/adapters/interactive_brokers/src/data/convert.rs
sed -n '1,260p' crates/adapters/interactive_brokers/src/historical/client.rs
sed -n '1,300p' crates/adapters/interactive_brokers/src/providers/instruments.rs
sed -n '1,260p' crates/adapters/interactive_brokers/src/providers/parse.rs
sed -n '840,1160p' crates/adapters/interactive_brokers/src/data/core.rs
sed -n '1320,1460p' crates/adapters/interactive_brokers/src/data/core.rs
sed -n '1,140p' crates/adapters/interactive_brokers/src/execution/core_orders.rs
sed -n '1,140p' crates/adapters/interactive_brokers/src/execution/transform/policy.rs
sed -n '1,180p' crates/adapters/interactive_brokers/src/execution/transform.rs
sed -n '360,420p' crates/adapters/interactive_brokers/src/execution/parse.rs
sed -n '1240,1310p' crates/adapters/interactive_brokers/src/providers/instruments.rs
sed -n '1860,2020p' crates/adapters/interactive_brokers/src/providers/instruments.rs
sed -n '1,220p' crates/adapters/interactive_brokers/tests/connection.rs
```

Required and final validation:

```bash
scripts/ai/verify_fast.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-016.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- `scripts/ai/verify_fast.sh`: passed; output ended with `== verify_fast complete ==`.
- Final JSON validation for task state and lease: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed with `agentflow role protocol validation passed`.
- `git diff --check`: passed.

## Tests Added Or Updated

No tests were added in RADP-016. This task is an inventory-only adapter parity
task. RADP-017 is the follow-up fixture task for the gaps recorded here.

## Behavior Impact

No runtime behavior changed. No Interactive Brokers market-data subscription,
historical request, instrument provider, order execution, account update,
Docker gateway, credential, public API, Python API, PyO3 binding, Cython
surface, Cargo feature, or persistence behavior changed.

The practical impact is release-gate clarity: Interactive Brokers adapter
parity gaps are now explicit follow-up inputs for fixture coverage and closure
decisions.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR documents adapter gaps without
changing runtime behavior or public APIs.

## Gate Status

RADP-016 is medium risk. It creates adapter inventory evidence only; it does not
change adapter runtime behavior or trading semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the inventory file, this evidence file, and the RADP-016 task state/lease
updates. No runtime, persisted data, adapter protocol, schema, Python, PyO3,
Cython, Cargo feature, or public API rollback is required.
