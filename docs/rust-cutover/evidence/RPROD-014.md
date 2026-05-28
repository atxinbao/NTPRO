# RPROD-014 Evidence

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-014

## Summary

Published the Rust product surface report for the RPROD phase. The report
summarizes current Rust CLI help contracts, Cargo example smokes, Rust API
documentation entrypoints, scope decisions, and owner-visible blockers that
must be handled by runtime, adapter, QA, removal, and release tasks.

## Files Changed

- `docs/rust-cutover/product/RUST_PRODUCT_SURFACE_REPORT.md`
- `docs/rust-cutover/evidence/RPROD-014.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RPROD-014.json`

## Commands Run

```bash
python3 scripts/ai/lease.py claim RPROD-014 --force --branch ai/RPROD-014-publish-rust-product-surface-report --agent-id Codex --path docs/rust-cutover/tasks/RPROD-014.md --path docs/rust-cutover/product/RUST_PRODUCT_SURFACE_REPORT.md --path docs/rust-cutover/evidence/RPROD-014.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-014.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_full.sh
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
python3 scripts/ai/lease.py release RPROD-014 --status PR_READY
```

## Command Results

- Lease claim succeeded for branch
  `ai/RPROD-014-publish-rust-product-surface-report`.
- Product surface report was added under `docs/rust-cutover/product/`.
- Required full verification passed with `scripts/ai/verify_full.sh`.
- Full verification included fast checks, `cargo fmt --check`,
  `cargo clippy -p nautilus-cli`, Rust test execution, golden trace schema
  validation, and Rust documentation generation.
- Golden trace validation reported:
  `valid trace: tests/golden/schema_smoke.jsonl (1 rows)`.
- Rust documentation generation completed and reported generated docs under
  `target/doc/nautilus_analysis/index.html` and 41 other files.
- Agentflow role validation passed with
  `python3 scripts/ai/validate_agentflow_roles.py`.
- Diff whitespace validation passed with `git diff --check`.
- Lease was released as `PR_READY`.

## Tests Added Or Updated

No Rust tests were added. This is a documentation and evidence task. The
required validation is the full verification script.

## Behavior Impact

- Documents the current Rust product surface and the remaining gaps.
- Does not change runtime behavior, trading semantics, adapter behavior,
  persistence, public Rust APIs, Python APIs, PyO3, or Cython paths.
- Does not approve Python, PyO3, Cython, adapter, release, or Rust-only removal
  work.

## Public API Impact

No public API change.

## Migration Note Status

No migration note required because this task adds documentation only.

## Rollback Plan

Revert the commit to remove the Rust product surface report, this evidence
file, and the RPROD-014 agentflow lease/status updates. No runtime or generated
data rollback is required.
