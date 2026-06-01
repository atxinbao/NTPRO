# RREM-004 Evidence - Map Python Workflows To Rust Workflows

Date: 2026-06-01
Executor: Codex
Task ID: RREM-004
Risk: medium
Release gatekeeper approval: user approved RREM-004/RREM-005 automation in this thread.

## Summary

Created a non-destructive Python-to-Rust workflow map. The map links Python
backtest, sandbox, live adapter, data/catalog, config, strategy, adapter tester,
notebook/debugging, and Python-only test workflows to current Rust replacement
paths or explicit blockers.

No Python, PyO3, Cython, runtime, adapter, Cargo behavior, packaging behavior,
or public API files were deleted or modified.

## Files Changed

- `docs/rust-cutover/migration/python_to_rust_workflow_map.md`
- `docs/rust-cutover/evidence/RREM-004.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-004.json`

## Commands Run

- `sed -n '1,220p' docs/rust-cutover/tasks/RREM-004.md`
- `sed -n '1,240p' docs/rust-cutover/product/RUST_CLI_CONTRACT.md`
- `sed -n '1,240p' docs/rust-cutover/product/BACKTEST_CLI_CONTRACT.md`
- `sed -n '1,220p' docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md`
- `sed -n '1,220p' docs/rust-cutover/product/DATA_CATALOG_CLI_CONTRACT.md`
- `sed -n '1,220p' docs/rust-cutover/product/CONFIG_VALIDATION_CLI_CONTRACT.md`
- `sed -n '1,220p' docs/rust-cutover/product/RUST_API_ENTRYPOINTS.md`
- `find examples -maxdepth 3 -type f \\( -name '*.py' -o -name '*.ipynb' \\) | sed -E 's#^examples/([^/]+)/.*#\\1#' | sort | uniq -c | sort -nr`
- `find examples/rust -type f | sed -E 's#^examples/rust/([^/]+)/.*#\\1#' | sort | uniq -c | sort -nr`
- `find tests -type f -name '*.py' | sed -E 's#^tests/([^/]+)/.*#\\1#' | sort | uniq -c | sort -nr`
- `cargo run -q -p nautilus-cli -- backtest --help`
- `cargo run -q -p nautilus-cli -- sandbox --help`
- `cargo run -q -p nautilus-cli -- live --help`
- `cargo run -q -p nautilus-cli -- data --help`
- `cargo run -q -p nautilus-cli -- config --help`
- `scripts/ai/verify_fast.sh`
- `python3 -m json.tool .agentflow/state/task_status.json`
- `python3 -m json.tool .agentflow/leases/RREM-004.json`
- `python3 scripts/ai/validate_agentflow_roles.py`
- `git diff --check`

## Command Results

- Python examples by top-level family: `live` 60, `backtest` 49, `sandbox` 8,
  `other` 4, `utils` 2.
- Rust examples/docs under `examples/rust`: backtest, sandbox, live, data,
  config, and root README entries exist.
- Python tests by top-level family: `unit_tests` 284, `integration_tests` 201,
  `performance_tests` 24, `mem_leak_tests` 18, `acceptance_tests` 3,
  `docs_tests` 2, plus root pytest files.
- CLI help passed for `backtest`, `sandbox`, `live`, `data`, and `config`.
- `scripts/ai/verify_fast.sh`: passed.
- JSON validation for state and lease files: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.

## Tests Added Or Updated

No tests were added or updated. This is a documentation and workflow-mapping
task.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No Python, PyO3, or
Cython product surfaces were removed.

## Public API Impact

None.

## Migration Note Status

No migration note required for this task because it maps workflows without
removing them. Later deletion tasks must add user-facing migration notes once
specific Python workflows are removed or archived.

## Rollback Plan

Revert `docs/rust-cutover/migration/python_to_rust_workflow_map.md`,
`docs/rust-cutover/evidence/RREM-004.md`, and the RREM-004 state/lease updates.
