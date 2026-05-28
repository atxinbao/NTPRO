# RTRACE-004 Evidence - Rust golden trace runner harness

Date: 2026-05-29
Executor: Codex
Task ID: RTRACE-004

## Summary

Added a Rust integration-test harness that validates all `tests/golden/*.jsonl`
fixtures against the `golden-trace-v1` schema contract, then bound that harness
to `scripts/ai/run_golden_traces.sh`.

## Files Changed

- `crates/testkit/tests/golden_trace_schema.rs`
- `scripts/ai/run_golden_traces.sh`
- `docs/rust-cutover/golden_trace/SCHEMA.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RTRACE-004.json`

## Commands Run

```bash
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-testkit --test golden_trace_schema
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/run_golden_traces.sh
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RTRACE-004.json >/dev/null
git diff --check
```

## Command Results

- `cargo test -p nautilus-testkit --test golden_trace_schema`: passed; 1 Rust
  golden trace schema test passed.
- `scripts/ai/run_golden_traces.sh`: passed; Python JSONL validation covered
  `market_data_schema.jsonl`, `order_lifecycle_schema.jsonl`, and
  `schema_smoke.jsonl`, then the Rust harness passed.
- `scripts/ai/verify_fast.sh`: passed after `cargo fmt`; fast mode ran toolchain
  detection and `cargo fmt --check`.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow` JSON validation: passed.
- `git diff --check`: passed.

## Tests Added or Updated

- Added `golden_trace_jsonl_files_match_rust_schema_contract` under
  `nautilus-testkit`.
- The Rust harness checks required row fields, schema version, case ID
  uniqueness per file, category allowlist, `input`/`expected` event arrays,
  event envelope fields, timestamp shape, payload object shape, and tolerances
  object shape.
- Updated `run_golden_traces.sh` so the standard golden trace command runs the
  Rust harness by default. `RUN_RUST_GOLDEN_TRACE_HARNESS=0` is reserved for
  documented local toolchain blockers.

## Behavior Impact

No trading runtime, adapter, Python, PyO3, or Cython behavior changed. The only
behavioral change is stricter local golden trace verification: the standard
trace command now fails if the Rust schema harness fails.

## Public API Impact

None.

## Migration Note Status

No migration note is required because no public API changed.

## Rollback Plan

Revert the Rust test file and the `run_golden_traces.sh` harness invocation.
The existing Python JSONL validator remains unchanged and can continue to
validate trace files independently.
