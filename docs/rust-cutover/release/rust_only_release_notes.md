# RREL-002 Rust-Only Release Notes

Date: 2026-06-03
Executor: Codex
Task ID: RREL-002 / RREL-008 / RREL-009

Updated: 2026-06-04
Executor: Codex
Follow-up ID: RC2-PRERELEASE-DOCS

Updated: 2026-06-04
Executor: Codex
Follow-up ID: FORMAL-RELEASE-V0.1.0

## Release State

This is the Rust-only release note package for the completed cutover record and
the release-candidate tag sequence.

RREL-009 made the local final release verification green, and the human owner
approved Rust-only cutover completion on 2026-06-03. RREL-008 records that
completion decision in documentation and agentflow state only.

After RREL-008 was reviewed and merged, the human owner separately approved the
first annotated release-candidate tag:

```text
ntpro-rust-only-rc.1
```

The tag points at commit `a886e2ac3682247b5e542599fb8dd219a6b9cf1c`. It is a
tag-only release candidate. No GitHub Release has been published.

After RC public-surface cleanup was reviewed and merged in PR #122, the human
owner approved the second annotated release-candidate tag:

```text
ntpro-rust-only-rc.2
```

This tag points at the commit containing the rc.2 tag-prep documentation and
the merged RC public-surface cleanup. It was published as a GitHub pre-release
and includes the Rust-only README, release documentation cleanup, and legacy
Python test removal.

```text
https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-rc.2
```

After RC2 pre-release documentation was corrected and reviewed in PR #125, the
human owner approved the third annotated release-candidate tag:

```text
ntpro-rust-only-rc.3
```

The tag points at commit `185f51dab9cf640d58f7b3956c4a6114f1e53d91`. It is the
final Rust-only pre-release source point before formal release publication.

```text
https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-rc.3
```

The formal Rust-only release is:

```text
ntpro-rust-only-v0.1.0
```

It is the first formal GitHub Release for the NTPRO Rust-only cutover track.

```text
https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.1.0
```

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
- `ntpro-rust-only-rc.1` records the first tag-only release candidate before
  RC public-surface cleanup.
- `ntpro-rust-only-rc.2` records the release candidate after RC public-surface
  cleanup and was published as a GitHub pre-release.
- `ntpro-rust-only-rc.3` records the final pre-release candidate after RC2
  documentation correction.
- `ntpro-rust-only-v0.1.0` records the first formal Rust-only GitHub Release.
- This RC cleanup removes the legacy Python test files under `tests/**/*.py`
  from the public release surface. The remaining Python files are local
  repository-control scripts under `scripts/`.

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

`ntpro-rust-only-v0.1.0` is the formal Rust-only GitHub Release after explicit
owner approval.

Before cutting a later GitHub Release, review the public README, release notes,
GitHub checks for the tagged commit, Rust CLI entrypoint evidence, and
repository language display.
