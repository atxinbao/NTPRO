# RTRACE-007 Evidence - Bind adapter payload trace fixtures

Date: 2026-05-29
Executor: Codex
Task ID: RTRACE-007

## Summary

Added an executable `adapter_payload` golden trace fixture for an OKX WebSocket
trade payload and bound it to the standard golden trace command through the
existing Rust OKX parser.

## Files Changed

- `tests/golden/adapter_payload_schema.jsonl`
- `crates/adapters/okx/tests/golden_trace_adapter_payload.rs`
- `docs/rust-cutover/evidence/RTRACE-007.md`
- `scripts/ai/run_golden_traces.sh`
- `docs/rust-cutover/golden_trace/SCHEMA.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RTRACE-007.json`

## Commands Run

```bash
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo test -p nautilus-okx --test golden_trace_adapter_payload
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/run_golden_traces.sh
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RTRACE-007.json >/dev/null
git diff --check
```

## Command Results

- `cargo test -p nautilus-okx --test golden_trace_adapter_payload`: passed; 1
  OKX adapter payload parser replay test passed.
- `scripts/ai/run_golden_traces.sh`: passed; validated 6 golden trace JSONL
  files, ran the Rust schema harness, ran the Rust backtest replay harness, ran
  the Rust live/sandbox lifecycle harness, and ran the OKX adapter payload
  harness.
- `scripts/ai/verify_fast.sh`: passed after `cargo fmt`; fast mode ran
  toolchain detection and `cargo fmt --check`.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `.agentflow` JSON validation: passed.
- `git diff --check`: passed.

## Tests Added or Updated

- Added `adapter_payload.okx_ws_trade.001` in
  `tests/golden/adapter_payload_schema.jsonl`.
- Added `okx_ws_trade_payload_replays_adapter_golden_trace` under
  `nautilus-okx`.
- Updated `run_golden_traces.sh` so the adapter payload harness runs by default.
  `RUN_RUST_ADAPTER_PAYLOAD_TRACE_REPLAY=0` is reserved for documented local
  toolchain or scoped replay blockers.

## Behavior Impact

No adapter implementation, runtime behavior, external API, secret handling,
Python, PyO3, or Cython surface changed. The verification surface is stricter
because the standard golden trace command now fails if the OKX adapter payload
parser harness fails.

## Public API Impact

None.

## Migration Note Status

No migration note is required because no public API changed.

## Rollback Plan

Revert the adapter payload fixture, the OKX Rust parser harness, and the
`run_golden_traces.sh` adapter payload invocation. The schema, backtest, and
live/sandbox lifecycle harnesses remain independently usable.
