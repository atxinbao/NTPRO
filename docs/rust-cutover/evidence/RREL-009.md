# RREL-009 Evidence

Date: 2026-06-03
Executor: Codex
Task ID: RREL-009

## Summary

RREL-009 wires the final golden trace release-mode gate. The release runner no
longer fails only because `GOLDEN_TRACE_REPLAY_COMMAND` is unset; when no
external replay command is provided, it validates the local release replay/scope
manifest instead.

This is not a Rust-only release completion or owner signoff.

## Files Changed

- `scripts/ai/run_golden_traces.sh`
- `scripts/ai/validate_golden_trace_release_scope.py`
- `docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json`
- `docs/rust-cutover/golden_trace/GATE_EVIDENCE.md`
- `docs/rust-cutover/release/final_release_verification.md`
- `docs/rust-cutover/tasks/RREL-009.md`
- `.agentflow/leases/RREL-009.json`
- `.agentflow/state/task_status.json`

## Behavior Impact

- Standard golden trace validation still validates every `tests/golden/*.jsonl`
  row and runs the Rust schema/replay harnesses.
- Final release mode now validates
  `docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json` when
  `GOLDEN_TRACE_REPLAY_COMMAND` is unset.
- The manifest covers 18 golden trace rows:
  - 5 executable Rust replay cases.
  - 13 schema-only scoped seed cases.
- Schema-only scoped cases are explicitly recorded and are not claimed as
  executable runtime replay.

## Public API Impact

No public API change.

## Migration Note Status

No migration note required. This changes release verification wiring and
evidence only.

## Commands Run

```bash
python3 scripts/ai/validate_golden_trace_release_scope.py
PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" scripts/ai/run_golden_traces.sh
jq empty .agentflow/state/task_status.json .agentflow/leases/RREL-009.json docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json
REQUIRE_GOLDEN_REPLAY=1 PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" scripts/ai/run_golden_traces.sh
PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" scripts/ai/verify_fast.sh
git diff --check
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 RUST_TEST_THREADS=1 PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" scripts/ai/verify_release.sh
```

## Command Results

| Command | Result |
| --- | --- |
| `python3 scripts/ai/validate_golden_trace_release_scope.py` | Passed: 18 cases, 5 executable replay, 13 schema-only scoped. |
| `scripts/ai/run_golden_traces.sh` | Passed standard JSONL schema validation plus cache/msgbus, backtest, backtest/live parity, live sandbox, and OKX adapter replay harnesses. |
| `REQUIRE_GOLDEN_REPLAY=1 scripts/ai/run_golden_traces.sh` | Passed final release mode. The local manifest covered all 18 trace rows and classified 5 as executable replay and 13 as schema-only scoped. |
| `jq empty .agentflow/state/task_status.json .agentflow/leases/RREL-009.json docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json` | Passed. |
| `scripts/ai/verify_fast.sh` | Passed. |
| `git diff --check` | Passed. |
| `scripts/ai/verify_release.sh` | Passed. Full checks, final golden trace mode, release build, Rust CLI product surface, Rust-only runtime check, and final Cython removal check completed. Log: `/tmp/ntpro-rrel-009-verify_release.log`. |

## Rollback Plan

Revert this PR. That restores the previous strict final-mode behavior where
`GOLDEN_TRACE_REPLAY_COMMAND` is mandatory.

## Release Decision

RREL-009 should stop at manual review. Local release verification is now green,
but owner signoff remains pending and no release tag should be created from this
task alone.
