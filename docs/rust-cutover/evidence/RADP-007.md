# RADP-007 Evidence - Inventory Rust Adapter Gaps For Coinbase BitMEX

Date: 2026-05-31
Executor: Codex
Task ID: RADP-007
Risk: medium

## Summary

Created the Coinbase and BitMEX Rust adapter gap inventory for RADP-007. The
inventory records current Rust data, execution, parser, fixture, and
Python/PyO3 boundary gaps, then classifies each surface as supported with
constraints, scoped out, partial, or deferred to removal gates.

No adapter runtime code changed. No parser, data-client, execution-client,
exchange protocol, credential, public API, Python/PyO3, Cython, Cargo feature,
or persistence behavior changed.

## Files Changed

- `docs/rust-cutover/inventory/coinbase_bitmex_adapter_gaps.md`
- `docs/rust-cutover/evidence/RADP-007.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-007.json`

## Commands Run

Context and inventory:

```bash
sed -n '1,260p' docs/rust-cutover/tasks/RADP-007.md
sed -n '1,220p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,220p' docs/rust-cutover/CONTRACT.md
sed -n '1,220p' docs/rust-cutover/DEFINITION_OF_DONE.md
sed -n '1,220p' docs/rust-cutover/AGENT_ROLES.md
rg --files crates/adapters/coinbase crates/adapters/bitmex
find crates/adapters/coinbase crates/adapters/bitmex -maxdepth 3 -type d
rg -n "TODO|FIXME|unimplemented|todo!|panic!|Unsupported|unsupported|not supported|Python|PyO3|Cython|demo|sandbox|credential|L3|order|execution|subscribe|websocket|WebSocket|REST|FIX|fixture|test_data" crates/adapters/coinbase crates/adapters/bitmex docs/rust-cutover -g '!target'
sed -n '1,220p' crates/adapters/coinbase/Cargo.toml
sed -n '1,220p' crates/adapters/coinbase/README.md
sed -n '1,240p' crates/adapters/bitmex/Cargo.toml
sed -n '1,240p' crates/adapters/bitmex/README.md
sed -n '1,260p' crates/adapters/coinbase/src/config.rs
sed -n '1,240p' crates/adapters/coinbase/src/factories.rs
sed -n '1,280p' crates/adapters/bitmex/src/config.rs
sed -n '1,260p' crates/adapters/bitmex/src/factories.rs
rg -n "Unsupported|unsupported|not support|requires|rejects|bail!|NoOrderSide|TimeInForce|OrderType|BookType|SubscribeMarkPrices|SubscribeIndexPrices|SubscribeFundingRates|SubscribeBookDepth10|SubscribeBookDeltas|request_bars|request_trades|request_book_snapshot" crates/adapters/coinbase/src/data/mod.rs crates/adapters/coinbase/src/execution.rs crates/adapters/coinbase/src/http/client.rs crates/adapters/coinbase/src/http/parse.rs crates/adapters/coinbase/src/provider.rs
rg -n "Unsupported|unsupported|not support|requires|rejects|bail!|NoOrderSide|TimeInForce|OrderType|BookType|SubscribeMarkPrices|SubscribeIndexPrices|SubscribeFundingRates|SubscribeBookDepth10|SubscribeBookDeltas|request_bars|request_trades|request_book_snapshot|deadman|broadcast" crates/adapters/bitmex/src/data.rs crates/adapters/bitmex/src/execution.rs crates/adapters/bitmex/src/http/client.rs crates/adapters/bitmex/src/http/parse.rs crates/adapters/bitmex/src/websocket/parse.rs crates/adapters/bitmex/src/websocket/client.rs
```

Counts:

```bash
for d in coinbase bitmex; do
  echo "$d test_files $(find crates/adapters/$d/tests -type f -name '*.rs' | wc -l | tr -d ' ')"
  echo "$d fixture_files $(find crates/adapters/$d/test_data -maxdepth 1 -type f | wc -l | tr -d ' ')"
  echo "$d annotated_tests $(rg -n '#\[test\]|#\[rstest\]|#\[tokio::test\]' crates/adapters/$d | wc -l | tr -d ' ')"
done
```

Required and final validation:

```bash
scripts/ai/verify_fast.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-007.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- `scripts/ai/verify_fast.sh`: passed; output ended with
  `== verify_fast complete ==`.
- Final JSON validation for task state and lease: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed with
  `agentflow role protocol validation passed`.
- `git diff --check`: passed.

## Tests Added Or Updated

No tests were added in RADP-007. This task is inventory-only and records the
gap list needed for RADP-008 fixture work.

## Behavior Impact

No runtime behavior changed. No parser, data-client, execution-client, exchange
protocol, credential handling, order routing, public API, Python API, PyO3
binding, Cython surface, Cargo feature, or persistence behavior changed.

The practical impact is release-gate clarity: Coinbase and BitMEX adapter gaps
are now visible before the fixture and closure tasks.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR documents adapter inventory
without changing runtime behavior or public APIs.

## Rollback Plan

Revert the RADP-007 inventory, evidence file, and task state/lease updates. No
runtime, persisted data, adapter protocol, schema, Python, PyO3, Cython, or
public API rollback is required.
