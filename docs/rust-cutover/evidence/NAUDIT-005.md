# NAUDIT-005 PostgreSQL Cache Adapter Classification Evidence

Date: 2026-06-05
Executor: Codex
Task ID: NAUDIT-005
Risk: medium

## 中文摘要

这次没有实现 PostgreSQL cache adapter，也没有改数据库 schema。

这次只是把产品状态说清楚：PostgreSQL cache adapter 在 v0.2 里是
`unsupported`，不能当成稳定的持久化 cache 产品路径。`nautilus database
init/drop` 仍然是数据库管理命令，但这不等于 PostgreSQL cache adapter 已经
支持完整 cache 持久化。

## Scope

Changed:

- `docs/rust-cutover/product/POSTGRES_CACHE_ADAPTER_STATUS.md`
- `docs/integrations/adapter_support_matrix.md`
- `docs/rust-cutover/verification/ignored_tests_risk_register.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/NAUDIT-005.json`

Created:

- `docs/rust-cutover/evidence/NAUDIT-005.md`

Not changed:

- No PostgreSQL schema changes.
- No cache implementation changes.
- No integration tests were unignored.
- No database CLI behavior changed.

## Classification Decision

PostgreSQL cache adapter status: `unsupported` for the v0.2 product surface.

Supported boundary that remains separate:

- `nautilus database init`;
- `nautilus database drop`;
- Rust in-memory cache paths.

Unsupported boundary:

- durable PostgreSQL cache persistence as a product path;
- PostgreSQL-backed cache release claims;
- ignored PostgreSQL cache tests as release evidence.

## Inventory Results

The source inventory found 21 explicit unsupported/not-implemented PostgreSQL
cache adapter paths in `crates/infrastructure/src/sql/cache.rs`, including:

- `load_synthetic`;
- `load_position`;
- `load_actor`;
- `delete_actor`;
- `load_strategy`;
- `delete_strategy`;
- `delete_order`;
- `delete_position`;
- `delete_account_event`;
- `add_synthetic`;
- `add_position`;
- `add_order_book`;
- `add_funding_rate`;
- `load_funding_rates`;
- `index_venue_order_id`;
- `index_order_position`;
- `update_position`;
- `snapshot_order_state`;
- `snapshot_position_state`;
- `heartbeat`.

PostgreSQL cache integration tests still blocked by schema/FK work:

- `test_order_cancel_rejected_insert_and_load`;
- `test_order_modify_rejected_insert_and_load`.

## Commands Run

```bash
rg -n "not implemented for PostgreSQL cache adapter|not implemented|bail!|todo!|unimplemented!" crates/infrastructure/src/sql/cache.rs
rg -n "PostgreSQL|postgres|cache adapter|cache persistence|database" docs/integrations docs/rust-cutover docs/getting_started docs/developer_guide README.md
python3 - <<'PY'
from pathlib import Path
import re
text=Path('crates/infrastructure/src/sql/cache.rs').read_text()
ops=[m.group(1) for m in re.finditer(r'"([^"\n]*not implemented for PostgreSQL cache adapter[^"\n]*)"', text)]
print('unsupported_count', len(ops))
for op in ops:
    print(op)
PY
git diff --check
scripts/ai/validate_agentflow_roles.py
scripts/ai/verify_fast.sh
```

## Results

- PostgreSQL cache source inventory completed.
- Unsupported/not-implemented source count: 21.
- Ignored PostgreSQL cache tests remain documented as schema/FK blockers.
- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`; as expected,
  it skipped workspace `cargo check`, clippy, release gate, and golden trace
  gate.

## Behavior Impact

No runtime behavior changed. This is support classification and documentation
only.

## Public API Impact

No public API changed.

## Migration Note Status

No migration note is required. The product boundary is documented in
`docs/rust-cutover/product/POSTGRES_CACHE_ADAPTER_STATUS.md`.

## Rollback Plan

Revert the NAUDIT-005 PR to remove the unsupported classification document and
restore the previous adapter/support matrix wording.
