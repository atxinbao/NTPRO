# RTRACE-002 Evidence

Date: 2026-05-29
Executor: Codex
Task ID: RTRACE-002

## Summary

Added market data golden trace fixtures using the `golden-trace-v1` schema.
The new fixture file covers representative market data events for quote ticks,
trade ticks, bars, order book deltas, instrument status transitions, and
catalog-style instrument-before-data ordering.

These fixtures are schema-validation evidence. They do not claim final Rust
runtime replay parity; later RTRACE tasks still own replay harness binding,
backtest/live replay, and adapter payload replay.

## Files Changed

- `tests/golden/market_data_schema.jsonl`
- `docs/rust-cutover/evidence/RTRACE-002.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RTRACE-002.json`

## Commands Run

```bash
python3 scripts/ai/lease.py claim RTRACE-002 --force --branch ai/RTRACE-002-capture-market-data-trace-fixtures --agent-id Codex --path docs/rust-cutover/tasks/RTRACE-002.md --path tests/golden/market_data_schema.jsonl --path docs/rust-cutover/evidence/RTRACE-002.md --path .agentflow/state/task_status.json --path .agentflow/leases/RTRACE-002.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/run_golden_traces.sh
python3 - <<'PY'
import json
from pathlib import Path
for path in [Path('tests/golden/market_data_schema.jsonl')]:
    with path.open() as f:
        for i, line in enumerate(f, 1):
            json.loads(line)
    print(f'ok jsonl {path}')
PY
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
python3 scripts/ai/lease.py release RTRACE-002 --status PR_READY
```

## Command Results

- Lease claim succeeded for branch
  `ai/RTRACE-002-capture-market-data-trace-fixtures`.
- Market data schema fixture was added under `tests/golden/`.
- Golden trace validation passed with `scripts/ai/run_golden_traces.sh`:
  `valid trace: tests/golden/market_data_schema.jsonl (6 rows)` and
  `valid trace: tests/golden/schema_smoke.jsonl (1 rows)`.
- JSONL parse validation passed for `tests/golden/market_data_schema.jsonl`.
- Required fast verification passed with `scripts/ai/verify_fast.sh`.
- Fast verification included toolchain discovery and `cargo fmt --check`.
- Optional fast cargo check and clippy remained intentionally skipped by the
  script defaults.
- Agentflow role validation passed with
  `python3 scripts/ai/validate_agentflow_roles.py`.
- Diff whitespace validation passed with `git diff --check`.
- Lease was released as `PR_READY`.

## Tests Added Or Updated

Added `tests/golden/market_data_schema.jsonl` with six
`golden-trace-v1` market data cases:

- `market_data.quote_tick.001`
- `market_data.trade_tick.001`
- `market_data.bar.001`
- `market_data.order_book_delta.001`
- `market_data.instrument_status.001`
- `market_data.catalog_ordering.001`

## Behavior Impact

- Adds schema-validation coverage for market data golden trace shape.
- Does not change runtime behavior, trading semantics, adapter behavior,
  persistence, public Rust APIs, Python APIs, PyO3, or Cython paths.
- Does not require external market data, credentials, or production venue
  access.

## Public API Impact

No public API change.

## Migration Note Status

No migration note required because this task adds test fixtures and evidence
only.

## Rollback Plan

Revert the commit to remove the market data golden trace fixture, this evidence
file, and the RTRACE-002 agentflow lease/status updates.
