# RTRACE-001 Evidence

Date: 2026-05-29
Executor: Codex
Task ID: RTRACE-001

## Summary

Defined the Rust golden trace v1 schema and required event categories. The
schema now documents required row fields, the event envelope, category registry,
tolerance rules, scope-blocker requirements, and validation commands. The
golden trace runner now validates the v1 schema fields instead of only checking
for `case_id`, `input`, and `expected`.

## Files Changed

- `docs/rust-cutover/golden_trace/SCHEMA.md`
- `scripts/ai/golden_trace_runner.py`
- `tests/golden/schema_smoke.jsonl`
- `docs/rust-cutover/evidence/RTRACE-001.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RTRACE-001.json`

## Commands Run

```bash
python3 scripts/ai/lease.py claim RTRACE-001 --force --branch ai/RTRACE-001-define-rust-golden-trace-schema --agent-id Codex --path docs/rust-cutover/tasks/RTRACE-001.md --path docs/rust-cutover/golden_trace/SCHEMA.md --path scripts/ai/golden_trace_runner.py --path tests/golden/schema_smoke.jsonl --path docs/rust-cutover/evidence/RTRACE-001.md --path .agentflow/state/task_status.json --path .agentflow/leases/RTRACE-001.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/run_golden_traces.sh
python3 -m py_compile scripts/ai/golden_trace_runner.py
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
python3 scripts/ai/lease.py release RTRACE-001 --status PR_READY
```

## Command Results

- Lease claim succeeded for branch
  `ai/RTRACE-001-define-rust-golden-trace-schema`.
- Golden trace schema documentation was expanded under
  `docs/rust-cutover/golden_trace/`.
- Golden trace runner validation was expanded to enforce `golden-trace-v1`
  schema fields, category values, event arrays, event timestamps, and payload
  objects.
- Golden trace validation passed with `scripts/ai/run_golden_traces.sh`:
  `valid trace: tests/golden/schema_smoke.jsonl (1 rows)`.
- Python compilation check passed for `scripts/ai/golden_trace_runner.py`.
- Required fast verification passed with `scripts/ai/verify_fast.sh`.
- Fast verification included toolchain discovery and `cargo fmt --check`.
- Optional fast cargo check and clippy remained intentionally skipped by the
  script defaults.
- Agentflow role validation passed with
  `python3 scripts/ai/validate_agentflow_roles.py`.
- Diff whitespace validation passed with `git diff --check`.
- Lease was released as `PR_READY`.

## Tests Added Or Updated

Updated `tests/golden/schema_smoke.jsonl` to use the v1 schema envelope and one
deterministic `market_data` event.

## Behavior Impact

- Golden trace validation now rejects stale or incomplete fixtures missing the
  v1 schema envelope.
- Does not change trading semantics, runtime behavior, adapter behavior,
  persistence, public Rust APIs, Python APIs, PyO3, or Cython paths.
- Does not add replay coverage for every category; later `RTRACE-*` tasks still
  own category-specific fixtures and replay binding.

## Public API Impact

No public API change.

## Migration Note Status

No migration note required because this task updates test evidence and
validation tooling only.

## Rollback Plan

Revert the commit to restore the previous minimal golden trace schema runner,
schema smoke fixture, schema documentation, evidence file, and RTRACE-001
agentflow lease/status updates.
