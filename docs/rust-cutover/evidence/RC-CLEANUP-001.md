# RC-CLEANUP-001 Evidence

Date: 2026-06-04
Executor: Codex
Task ID: RC-CLEANUP-001

## Plain Summary

This PR cleans up the public release-candidate surface after
`ntpro-rust-only-rc.1` was created.

In plain terms: the repository was already approved as Rust-only, but the public
README and several release documents still looked like the old mixed
Rust/Python project. The top-level Python tests also made GitHub still display
Python as a large part of the repository. This cleanup makes the public surface
match the Rust-only direction.

## Scope

- Rewrite the top-level README as an NTPRO Rust-only release-candidate entry.
- Update release documents to record that `ntpro-rust-only-rc.1` exists as a
  tag-only release candidate.
- Record that no GitHub Release has been published.
- Remove tracked legacy Python tests under `tests/**/*.py`.
- Keep local Python helper scripts under `scripts/` because they are
  repository-control and release-evidence automation, not product APIs.

## Files Changed

- `README.md`
- `docs/rust-cutover/release/*.md`
- `docs/rust-cutover/migration/python_test_scope_map.md`
- `docs/rust-cutover/migration/python_to_rust_workflow_map.md`
- `docs/rust-cutover/migration/python_package_surface_removal_stage.md`
- `tests/**/*.py`

## Behavior Impact

No Rust runtime behavior changes.

The legacy Python test suite is removed from the release product surface. Rust
crate tests, golden traces, adapter fixture evidence, and release verification
remain the active validation path.

## Public API Impact

No Rust public API change.

Python package/import APIs remain unsupported for the Rust-only release
candidate.

## Migration Note Status

Migration notes are updated to record the legacy Python test cleanup decision.

## Commands Run

```bash
git ls-files | rg '\.py$|(^|/)pyproject\.toml$|(^|/)uv\.lock$|(^|/)requirements.*\.txt$'
git diff --check
git diff --cached --check
rg -n 'The Rust-only release is not ready|Users should not treat|must still not be tagged|Do not create a release candidate tag|No release candidate tag is created|Release candidate tag \| Not created|Current decision: blocked|still block the final|does not delete the remaining Python tests|do not bulk-delete|RREL-008 must still be reviewed|owner signoff remains pending|check_rust_only_runtime\.sh.*still fails' README.md docs/rust-cutover/release docs/rust-cutover/migration
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_cython_removed.sh
scripts/ai/verify_fast.sh
gh run list --repo atxinbao/NTPRO --limit 8 --json databaseId,name,displayTitle,status,conclusion,headBranch,headSha,event,url,createdAt
```

## Command Results

| Command | Result |
| --- | --- |
| residual Python file scan | Passed. Remaining tracked Python files: 7, all under `scripts/`; tracked Python tests: 0. Root `pyproject.toml` and `uv.lock` remain as local tooling metadata. |
| tracked language-size estimate | Rust `37,518,803` bytes; Python `37,801` bytes; Shell `325,648` bytes; TOML `230,538` bytes. |
| `git diff --check` | Passed. |
| `git diff --cached --check` | Passed. |
| contradiction scan | Passed. No current-state wording found for old "not ready", "tag not created", or "blocked" release claims in README/release/migration docs. |
| `scripts/ai/check_rust_only_runtime.sh` | Passed: `== rust-only-runtime: ok ==`. |
| `scripts/ai/check_cython_removed.sh` | Passed: `== cython-removed: ok ==`. |
| `scripts/ai/verify_fast.sh` | Passed. Rust fmt completed; cargo check and clippy were skipped by current fast-check defaults unless their environment flags are enabled. |
| GitHub run list | Current main nightly runs for commit `a886e2ac3682247b5e542599fb8dd219a6b9cf1c` are still queued; prior RREL-008 and RREL-009 PR smoke checks are successful. |

## Rollback Plan

Revert this PR. That restores the old README/release docs and the legacy Python
tests under `tests/**/*.py`.
