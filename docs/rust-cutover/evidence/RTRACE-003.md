# RTRACE-003 Evidence

Date: 2026-05-29
Executor: Codex
Task ID: RTRACE-003

## Summary

Added order lifecycle golden trace fixtures using the `golden-trace-v1`
schema. The new fixture file covers representative command/event flows for
submit acceptance, submit rejection, modify acceptance, cancel acceptance,
triggered stop fill, and partial-fill-to-filled sequencing.

These fixtures are schema-validation evidence. They do not claim final Rust
runtime replay parity; later RTRACE tasks still own replay harness binding and
runtime replay execution.

## Files Changed

- `tests/golden/order_lifecycle_schema.jsonl`
- `docs/rust-cutover/evidence/RTRACE-003.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RTRACE-003.json`

## Commands Run

```bash
python3 scripts/ai/lease.py claim RTRACE-003 --force --branch ai/RTRACE-003-capture-order-lifecycle-trace-fixtures --agent-id Codex --path docs/rust-cutover/tasks/RTRACE-003.md --path tests/golden/order_lifecycle_schema.jsonl --path docs/rust-cutover/evidence/RTRACE-003.md --path .agentflow/state/task_status.json --path .agentflow/leases/RTRACE-003.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/run_golden_traces.sh
python3 - <<'PY'
import json
from pathlib import Path
for path in [Path('tests/golden/order_lifecycle_schema.jsonl')]:
    with path.open() as f:
        for i, line in enumerate(f, 1):
            json.loads(line)
    print(f'ok jsonl {path}')
PY
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
python3 scripts/ai/lease.py release RTRACE-003 --status PR_READY
```

## Command Results

- Lease claim succeeded for branch
  `ai/RTRACE-003-capture-order-lifecycle-trace-fixtures`.
- Order lifecycle schema fixture was added under `tests/golden/`.
- Golden trace validation passed with `scripts/ai/run_golden_traces.sh`:
  `valid trace: tests/golden/market_data_schema.jsonl (6 rows)`,
  `valid trace: tests/golden/order_lifecycle_schema.jsonl (6 rows)`, and
  `valid trace: tests/golden/schema_smoke.jsonl (1 rows)`.
- JSONL parse validation passed for
  `tests/golden/order_lifecycle_schema.jsonl`.
- Required fast verification passed with `scripts/ai/verify_fast.sh`.
- Fast verification included toolchain discovery and `cargo fmt --check`.
- Optional fast cargo check and clippy remained intentionally skipped by the
  script defaults.
- Agentflow role validation passed with
  `python3 scripts/ai/validate_agentflow_roles.py`.
- Diff whitespace validation passed with `git diff --check`.
- Lease was released as `PR_READY`.

## Tests Added Or Updated

Added `tests/golden/order_lifecycle_schema.jsonl` with six
`golden-trace-v1` order lifecycle cases:

- `order_lifecycle.submit_accept.001`
- `order_lifecycle.submit_reject.001`
- `order_lifecycle.modify_accept.001`
- `order_lifecycle.cancel_accept.001`
- `order_lifecycle.triggered_fill.001`
- `order_lifecycle.partial_then_filled.001`

## Behavior Impact

- Adds schema-validation coverage for order lifecycle golden trace shape.
- Does not change runtime behavior, trading semantics, adapter behavior,
  persistence, public Rust APIs, Python APIs, PyO3, or Cython paths.
- Does not require external venue access or credentials.

## Public API Impact

No public API change.

## Migration Note Status

No migration note required because this task adds test fixtures and evidence
only.

## Rollback Plan

Revert the commit to remove the order lifecycle golden trace fixture, this
evidence file, and the RTRACE-003 agentflow lease/status updates.
