# RREL-002 Rust-Only Release Notes

Date: 2026-06-03
Executor: Codex
Task ID: RREL-002 / RREL-008 / RREL-009

## Release State

This is the Rust-only release note package for the completed cutover record.

RREL-009 made the local final release verification green, and the human owner
approved Rust-only cutover completion on 2026-06-03. RREL-008 records that
completion decision in documentation and agentflow state only. It does not
create a release candidate tag, publish a GitHub Release, or enable auto-merge.

## What Changed In This Release Track

- Rust product, runtime, adapter, trace, removal, and release evidence has been
  collected under `docs/rust-cutover/evidence/`.
- Migration notes describe the Rust-only path and the removed Python/PyO3/Cython
  product surfaces.
- Removal evidence through `RREM-022` records the staged cleanup of Python,
  PyO3, Cython, Cap'n Proto, and residual Rust crate references.
- RREL-009 records the final golden trace release-mode scope and green
  `verify_release.sh` result.
- RREL-008 records the human owner completion approval.

## Breaking Changes

The Rust-only cutover intentionally removes the legacy Python product surface
from the v2 release track:

- Python product package/import surfaces are no longer the release product
  path.
- PyO3 bridge crates and per-crate binding surfaces have been removed from the
  Rust-only product path.
- Cython source/interface files and active Cython build references have been
  removed from the Rust-only product path.
- Python/Cython packaging assumptions have been removed from the final
  Rust-only release gate.
- Cap'n Proto serialization support was removed as part of the cutover cleanup
  because it was not part of the Rust-only release target.

## Replacement Workflows

| Legacy workflow | Replacement workflow | Current note |
| --- | --- | --- |
| Python package usage | Rust CLI/API/docs/examples | Rust-only product path. |
| PyO3 bridge usage | Native Rust product/runtime access | Removed from release product path. |
| Cython implementation/build path | Rust workspace build and tests | Removed from release product path. |
| Mixed Python release packaging | Rust-only release gate | Final local release verification passed in RREL-009. |

## Validation Summary

RREL-009 passed:

- `scripts/ai/verify_release.sh`
- `scripts/ai/check_rust_only_runtime.sh`
- `scripts/ai/check_cython_removed.sh`
- `scripts/ai/run_golden_traces.sh`
- `REQUIRE_GOLDEN_REPLAY=1 scripts/ai/run_golden_traces.sh`

RREL-008 is documentation/state-only and records the owner-approved completion
decision. It must still be reviewed and merged before the agentflow state can be
closed as `DONE`.

## Release Recommendation

Do not tag or publish a Rust-only GitHub Release from RREL-008.

After the RREL-008 completion PR is reviewed and merged, the next release action
must be a separate owner-approved tag/release procedure. The draft procedure is
recorded in `docs/rust-cutover/release/release_candidate_tag_plan.md`.
