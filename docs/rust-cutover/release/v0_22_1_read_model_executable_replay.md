# v0.22.1 Read-model Executable Replay Expansion

Date: 2026-07-02
Executor: Codex
Task: `V221-003` / GitHub issue `#707`
Status: LOCAL VALIDATION PASSED

## Summary

v0.22.1 expands Trader Terminal Workbench read-model executable replay coverage.
The Rust projection harness now replays 28 read-model rows, including positions,
fills, order mismatch/duplicate cases, risk states, and dashboard forbidden
controls.

Plain Chinese summary: v0.22.1 增加 read-model executable replay 覆盖，但不把
版本表述成完整 executable read-model runtime。当前定位仍是 Workbench/runtime bridge；
contract baseline 和 account missing/redaction 仍是 schema-only scope。

## Release Scope

```text
read_model executable_replay = 28
read_model schema_only_scoped = 4
all manifest cases = 83
all executable_replay cases = 78
all schema_only_scoped cases = 5
```

Promoted families:

```text
position rows
fill execution rows
order unknown/readback mismatch/duplicate rows
risk visible/manual_review/halted/stale rows
dashboard forbidden-controls row
```

## Remaining Boundary

The following read-model rows remain schema-only after V221-003:

```text
read_model.contract.healthy_minimal.001
read_model.contract.fail_closed_missing_lineage_source_freshness.001
read_model.account_snapshot.missing_provenance.001
read_model.account_snapshot.redaction_breach.001
```

## Validation

```text
cargo fmt --all -- --check = PASS
python3 scripts/ai/validate_golden_trace_release_scope.py --manifest docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json --trace-glob 'tests/golden/*.jsonl' = PASS, 83 cases, 78 executable replay, 5 schema-only scoped
cargo test -p nautilus-cli --test golden_trace_read_model_projection -- --nocapture = PASS, 1 test passed
scripts/ai/verify_fast.sh = PASS, fast smoke only
```
