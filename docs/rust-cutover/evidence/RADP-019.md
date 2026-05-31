# RADP-019 Evidence - Inventory Rust Adapter Gaps For Betfair Architect AX

Date: 2026-05-31
Executor: Codex
Task ID: RADP-019
Risk: medium

## Summary

Inventoried the Betfair and Architect AX Rust adapter surfaces and recorded
the remaining Rust adapter parity gaps for the follow-up fixture and closure
tasks.

No adapter runtime behavior changed. This task only adds the inventory document
that classifies current Rust data, execution, parser, fixture, operational, and
Python/PyO3 boundary gaps.

## Files Changed

- `docs/rust-cutover/inventory/betfair_architect_ax_adapter_gaps.md`
- `docs/rust-cutover/evidence/RADP-019.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-019.json`

## Commands Run

```bash
sed -n '1,220p' docs/rust-cutover/tasks/RADP-019.md
sed -n '1,260p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,260p' docs/rust-cutover/AGENT_ROLES.md
sed -n '1,260p' .agentflow/roles.yaml
sed -n '1,260p' .agentflow/policies/path_scope.yaml
find crates/adapters/betfair/test_data -type f | wc -l
find crates/adapters/architect_ax/test_data -type f | wc -l
python3 - <<'PY'
import pathlib, re
for adapter in ["betfair", "architect_ax"]:
    count = 0
    for path in pathlib.Path(f"crates/adapters/{adapter}").rglob("*.rs"):
        count += len(re.findall(r"#\[(?:tokio::test|test|rstest)", path.read_text(errors="ignore")))
    print(adapter, count)
PY
rg -n "not supported|unsupported|downgrad|BookType|Index prices|Instrument close|GTD|Market" crates/adapters/architect_ax/src/data.rs crates/adapters/architect_ax/src/execution.rs crates/adapters/architect_ax/src/common/enums.rs crates/adapters/architect_ax/src/websocket/orders/client.rs
rg -n "not supported|unsupported|Python|pyo3|custom_data|BookType|subscribe_|reconnect|keep alive|keepalive|MarketOnClose|LimitOnClose|MarketToLimit" crates/adapters/betfair/src/data.rs crates/adapters/betfair/src/execution.rs crates/adapters/betfair/src/data_types.rs crates/adapters/betfair/src/lib.rs crates/adapters/betfair/README.md
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-019.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
scripts/ai/verify_fast.sh
git diff --check
```

## Command Results

- Betfair fixture count: 71 files under `crates/adapters/betfair/test_data/`.
- Architect AX fixture count: 52 files under `crates/adapters/architect_ax/test_data/`.
- Betfair Rust test annotation count: 511.
- Architect AX Rust test annotation count: 380.
- Repository inspection confirmed Betfair and Architect AX Rust data,
  execution, fixture, reconnect/auth, unsupported-surface, and Python/PyO3
  boundary evidence used in the inventory.
- Final JSON validation, agentflow role validation, `verify_fast.sh`, and
  `git diff --check`: passed.

## Tests Added Or Updated

None. RADP-019 is an inventory/evidence task and does not change runtime code or
test code.

## Behavior Impact

No runtime behavior, adapter behavior, public API, trading semantics, Python,
PyO3, Cython, or Cargo feature behavior changed. The inventory only records
support classifications and follow-up gaps for RADP-020/RADP-021.

## Public API Impact

None.

## Migration Note Status

No migration note required because there is no public API or runtime behavior
change.

## Rollback Plan

Revert the Betfair/Architect AX adapter gap inventory, this evidence file, and
the RADP-019 agentflow state and lease updates.
