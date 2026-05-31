# RADP-004 Evidence - Inventory Rust Adapter Gaps For Bybit OKX Kraken

Date: 2026-05-31
Executor: Codex
Task ID: RADP-004
Risk: medium

## Summary

Inventoried the current Rust adapter gaps for Bybit, OKX, and Kraken. The new
inventory records each adapter's Rust product boundary, fixture/test coverage,
known supported-with-constraints surfaces, deferred Python/PyO3 bridge surfaces,
and follow-up scope for RADP-005/RADP-006.

This task does not implement new adapter behavior. It creates the release-gate
input needed before fixture expansion and gap closure.

## Files Changed

- `docs/rust-cutover/inventory/bybit_okx_kraken_adapter_gaps.md`
- `docs/rust-cutover/evidence/RADP-004.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RADP-004.json`

## Commands Run

Task setup and context:

```bash
git status --short --branch
sed -n '1,220p' docs/rust-cutover/tasks/RADP-004.md
sed -n '1,260p' docs/rust-cutover/inventory/binance_adapter_gaps.md
sed -n '1,220p' docs/rust-cutover/TASK_EXECUTION.md
sed -n '1,240p' .agentflow/roles.yaml
sed -n '1,220p' .agentflow/policies/path_scope.yaml
sed -n '1,220p' .agentflow/leases/RADP-004.json
```

Adapter inventory:

```bash
find crates/adapters/bybit crates/adapters/okx crates/adapters/kraken -maxdepth 2 -type f
rg -n "struct .*Config|enum .*Product|ProductType|Factory|impl .*Factory|cfg\\(feature = \\\"python\\\"\\)|pyo3|panic|bail!|unsupported|not supported|TODO|FIXME" crates/adapters/bybit crates/adapters/okx crates/adapters/kraken
find crates/adapters/bybit/test_data crates/adapters/okx/test_data crates/adapters/kraken/test_data -maxdepth 3 -type f
find crates/adapters/bybit/tests crates/adapters/okx/tests crates/adapters/kraken/tests -maxdepth 2 -type f
sed -n '1,260p' crates/adapters/bybit/src/config.rs
sed -n '1,260p' crates/adapters/bybit/src/factories.rs
sed -n '260,360p' crates/adapters/bybit/src/common/enums.rs
sed -n '760,1090p' crates/adapters/bybit/src/data.rs
sed -n '1,280p' crates/adapters/okx/src/config.rs
sed -n '1,280p' crates/adapters/okx/src/factories.rs
sed -n '250,340p' crates/adapters/okx/src/common/enums.rs
sed -n '1030,1095p' crates/adapters/okx/src/data.rs
sed -n '240,285p' crates/adapters/okx/src/execution.rs
sed -n '1,280p' crates/adapters/kraken/src/config.rs
sed -n '1,280p' crates/adapters/kraken/src/factories.rs
sed -n '88,110p' crates/adapters/kraken/src/common/enums.rs
sed -n '185,218p' crates/adapters/kraken/src/data/spot.rs
sed -n '1,240p' docs/integrations/bybit.md
sed -n '1,260p' docs/integrations/okx.md
sed -n '1,260p' docs/integrations/kraken.md
```

Tool availability:

```bash
tool_search: code-index MCP code search repository symbols
```

Required and final validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_fast.sh
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RADP-004.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
```

## Command Results

- Local task and role context reads: passed.
- Adapter inventory scans: passed.
- `tool_search` exposed GitHub code search but no local `code-index` tool for
  this session, so repository inspection used local `rg`, `find`, and targeted
  file reads.
- Required Rust 1.95 `scripts/ai/verify_fast.sh`: passed and ended with
  `== verify_fast complete ==`.
- `verify_fast.sh` skipped the legacy mixed-workspace cargo check and clippy by
  default, matching the script's current fast-mode behavior.
- Final JSON validation for task state and lease: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.

## Tests Added Or Updated

No Rust tests were added or updated. RADP-004 is an inventory-only task. The
next fixture task, RADP-005, is the appropriate place to add executable adapter
fixture coverage for the recorded gaps.

## Behavior Impact

No runtime behavior changed. No trading semantics, parser behavior, order
behavior, exchange protocol handling, credential handling, public API, Python
API, PyO3 binding, Cython surface, Cargo feature behavior, or persistence format
changed.

The practical impact is release-gate clarity: Bybit, OKX, and Kraken now have
a concrete Rust adapter gap list that RADP-005 and RADP-006 can validate and
close.

## Public API Impact

No public API change.

## Migration Note Status

No migration note is required because this PR does not change public API,
runtime behavior, persisted data, or user-facing configuration semantics.

## Gate Status

RADP-004 is medium risk. It is documentation and inventory work only; it does
not change adapter runtime behavior or trading semantics.

Auto-merge is allowed after local validation and required PR checks pass.

## Rollback Plan

Revert the Bybit/OKX/Kraken adapter inventory file, this evidence file, and
the RADP-004 task state/lease updates. No runtime, persisted data, adapter
protocol, schema, Python, PyO3, Cython, or public API rollback is required.
