# RADP-022 Evidence - Inventory Rust Adapter Gaps For Polymarket Sandbox

Date: 2026-05-31
Executor: Codex
Task ID: RADP-022
Risk: medium

## Summary

Created the Polymarket and Sandbox Rust adapter gap inventory for the R5
adapter parity sequence. The inventory records current Rust surfaces,
fixture/test coverage, support classifications, and release-gate gaps for
RADP-023 and RADP-024.

The task is documentation and control-state only. It does not change adapter
runtime behavior, market-data handling, order execution, account handling,
sandbox matching behavior, credential handling, public APIs, Python/PyO3
bindings, Cython surfaces, or Cargo feature behavior.

## Files Changed

- `docs/rust-cutover/inventory/polymarket_sandbox_adapter_gaps.md`
- `docs/rust-cutover/evidence/RADP-022.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-022.json`

## Commands Run

Context and inventory:

```bash
sed -n '1,260p' docs/rust-cutover/tasks/RADP-022.md
sed -n '1,220p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,220p' docs/rust-cutover/CONTRACT.md
sed -n '1,220p' docs/rust-cutover/DEFINITION_OF_DONE.md
sed -n '1,260p' docs/rust-cutover/AGENT_ROLES.md
sed -n '1,260p' docs/rust-cutover/inventory/interactive_brokers_adapter_gaps.md
sed -n '1,260p' docs/rust-cutover/inventory/betfair_architect_ax_adapter_gaps.md
find crates/adapters/polymarket crates/adapters/sandbox -maxdepth 3 -type f | sort
sed -n '1,260p' crates/adapters/polymarket/README.md
sed -n '1,260p' crates/adapters/polymarket/Cargo.toml
sed -n '1,220p' crates/adapters/sandbox/README.md
sed -n '1,220p' crates/adapters/sandbox/Cargo.toml
python3 - <<'PY'
import pathlib,re
for adapter in ['polymarket','sandbox']:
    count=0
    for path in pathlib.Path(f'crates/adapters/{adapter}').rglob('*.rs'):
        count += len(re.findall(r'#\[(?:tokio::test|test|rstest)', path.read_text(errors='ignore')))
    files=sum(1 for _ in pathlib.Path(f'crates/adapters/{adapter}').rglob('*.rs'))
    fixtures=sum(1 for _ in pathlib.Path(f'crates/adapters/{adapter}/test_data').rglob('*') if _.is_file()) if pathlib.Path(f'crates/adapters/{adapter}/test_data').exists() else 0
    print(adapter, 'rs_files', files, 'test_annotations', count, 'fixtures', fixtures)
PY
rg -n 'bail!|unsupported|Unsupported|not supported|does not support|TODO|todo|warn!|error!' crates/adapters/polymarket/src crates/adapters/polymarket/tests
rg -n 'bail!|unsupported|Unsupported|not supported|does not support|TODO|todo|warn!|error!' crates/adapters/sandbox/src crates/adapters/sandbox/tests
rg -n 'pyclass|pyo3|feature = "python"|cfg\(feature = "python"\)|extension-module|python' crates/adapters/polymarket/src crates/adapters/polymarket/Cargo.toml crates/adapters/sandbox/src crates/adapters/sandbox/Cargo.toml
rg -n 'Polymarket|sandbox|Sandbox' docs/integrations docs/rust-cutover docs -g'*.md'
```

Targeted source reads:

```bash
sed -n '1,260p' crates/adapters/polymarket/src/config.rs
sed -n '1234,1565p' crates/adapters/polymarket/src/data.rs
sed -n '950,1320p' crates/adapters/polymarket/src/execution/mod.rs
sed -n '1380,1805p' crates/adapters/polymarket/src/execution/mod.rs
sed -n '1,280p' crates/adapters/polymarket/src/execution/order_builder.rs
sed -n '1,220p' crates/adapters/polymarket/src/common/enums.rs
sed -n '1,240p' crates/adapters/sandbox/src/config.rs
sed -n '1,240p' crates/adapters/sandbox/src/execution.rs
sed -n '760,1085p' crates/adapters/sandbox/src/execution.rs
sed -n '1,200p' docs/rust-cutover/tasks/RADP-023.md
sed -n '1,200p' docs/rust-cutover/tasks/RADP-024.md
```

Required and final validation:

```bash
scripts/ai/verify_fast.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-022.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- Polymarket scan: 63 Rust files, 647 annotated tests, and 45 checked-in
  `test_data/` fixtures.
- Sandbox scan: 9 Rust files, 40 annotated tests, and no checked-in
  `test_data/` fixture directory.
- Repository inspection confirmed Polymarket and Sandbox Rust data,
  execution, fixture, matching, operational, unsupported-surface, and
  Python/PyO3 boundary evidence used in the inventory.
- `scripts/ai/verify_fast.sh`: passed with `== verify_fast complete ==`.
- Final JSON validation, agentflow role validation, and `git diff --check`:
  passed.

## Tests Added Or Updated

No tests were added in RADP-022. This task is an inventory-only adapter parity
task. RADP-023 is the follow-up fixture task for the gaps recorded here.

## Behavior Impact

No runtime behavior changed. No Polymarket market-data subscription, order
execution, wallet signing, account query, reconciliation, Sandbox matching,
Sandbox account handling, credential, public API, Python API, PyO3 binding,
Cython surface, Cargo feature, or persistence behavior changed.

The practical impact is release-gate clarity: Polymarket and Sandbox adapter
parity gaps are now explicit follow-up inputs for fixture coverage and closure
decisions.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR documents adapter gaps without
changing runtime behavior or public APIs.

## Gate Status

RADP-022 is medium risk. It creates adapter inventory evidence only; it does
not change adapter runtime behavior or trading semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the inventory file, this evidence file, and the RADP-022 task
state/lease updates. No runtime, persisted data, adapter protocol, matching
behavior, schema, Python, PyO3, Cython, Cargo feature, or public API rollback is
required.
