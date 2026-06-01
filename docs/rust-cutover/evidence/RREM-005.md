# RREM-005 Evidence - Port Or Scope Python-Only Tests

Date: 2026-06-01
Executor: Codex
Task ID: RREM-005
Risk: medium
Release gatekeeper approval: user approved RREM-004/RREM-005 automation in this thread.

## Summary

Created a non-destructive Python test scope map. The map classifies Python
unit, integration, adapter, performance, memory leak, acceptance, docs, and
package-overlay tests into port/replace/defer decisions for later removal work.

No Python tests were deleted, skipped, weakened, or rewritten.

## Files Changed

- `docs/rust-cutover/migration/python_test_scope_map.md`
- `docs/rust-cutover/evidence/RREM-005.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-005.json`

## Commands Run

- `sed -n '1,220p' docs/rust-cutover/tasks/RREM-005.md`
- `find tests -type f -name '*.py' | wc -l`
- `find python/tests -type f -name '*.py' | wc -l`
- `rg -l 'nautilus_pyo3|pyo3|PyO3|Cython|cython|as_legacy_cython|\\.pyx|\\.pxd' tests python/tests -g '*.py' | wc -l`
- `find crates -path '*/tests/*.rs' -type f | wc -l`
- `find crates -path '*/examples/*.rs' -type f | wc -l`
- `rg -l 'golden_trace|Golden|trace' crates -g '*.rs' | wc -l`
- `find tests -type f -name '*.py' | sed -E 's#^tests/([^/]+)/.*#\\1#' | sort | uniq -c | sort -nr`
- `find python/tests -type f -name '*.py' | sed -E 's#^python/tests/([^/]+)/.*#python/tests/\\1#' | sort | uniq -c | sort -nr`
- `find crates -path '*/tests/*.rs' -type f | sed -E 's#^crates/([^/]+)/.*#\\1#' | sort | uniq -c | sort -nr`
- `rg -l 'nautilus_pyo3|pyo3|PyO3|Cython|cython|as_legacy_cython|\\.pyx|\\.pxd' tests python/tests -g '*.py'`
- `scripts/ai/verify_fast.sh`
- `python3 -m json.tool .agentflow/state/task_status.json`
- `python3 -m json.tool .agentflow/leases/RREM-005.json`
- `python3 scripts/ai/validate_agentflow_roles.py`
- `git diff --check`

## Command Results

- Python test files under `tests/**`: 534.
- Python package overlay test files under `python/tests/**`: 90.
- Python tests mentioning PyO3/Cython/legacy interop: 179.
- Rust crate test files under `crates/**/tests/*.rs`: 156.
- Rust crate examples under `crates/**/examples/*.rs`: 84.
- Rust files mentioning golden trace/parity/fixture terms: 97.
- Largest Python test families: `unit_tests` 284, `integration_tests` 201,
  `performance_tests` 24, `mem_leak_tests` 18.
- Largest Python domain groups: `tests/integration_tests/adapters` 185,
  `tests/unit_tests/model` 64, `tests/unit_tests/indicators` 39,
  `tests/unit_tests/backtest` 29, `tests/unit_tests/analysis` 23,
  `tests/unit_tests/common` 21.
- Largest Rust test family: adapter crates with 92 files.
- `scripts/ai/verify_fast.sh`: passed.
- JSON validation for state and lease files: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- `git diff --check`: passed.

## Tests Added Or Updated

No tests were added, deleted, skipped, or updated. This task records scope
decisions for later test port/removal work.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No Python, PyO3, or
Cython product surfaces were removed.

## Public API Impact

None.

## Migration Note Status

No migration note required for this task because it does not remove tests or
product surfaces. Later removal tasks must add migration notes when Python
test-backed user workflows or import paths are removed.

## Rollback Plan

Revert `docs/rust-cutover/migration/python_test_scope_map.md`,
`docs/rust-cutover/evidence/RREM-005.md`, and the RREM-005 state/lease updates.
