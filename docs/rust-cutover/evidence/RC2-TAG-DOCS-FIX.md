# RC2-TAG-DOCS-FIX Evidence

Date: 2026-06-04
Executor: Codex
Task ID: RC2-TAG-DOCS-FIX

## Plain Summary

This PR fixes a documentation self-reference issue before creating
`ntpro-rust-only-rc.2`.

In plain terms: PR #123 prepared the docs for rc.2, but after GitHub merged it,
the merge commit changed. Any hard-coded rc.2 commit hash in the docs would be
stale before the tag was created. This PR removes the fixed hash wording and
states that rc.2 points at the commit containing the rc.2 tag-prep docs and the
RC public-surface cleanup.

## Scope

- Remove hard-coded rc.2 commit hash references from release/migration docs.
- Keep rc.1 historical commit references unchanged.
- Do not create a tag in this PR.
- Do not publish a GitHub Release.
- Do not modify Rust runtime or trading code.

## Commands Run

```bash
rg -n "6445e4f6bf6bde69eae91596d4ff7644d2e41fc0|6445e4f" README.md docs/rust-cutover
git diff --check
PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" scripts/ai/check_rust_only_runtime.sh
PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" scripts/ai/check_cython_removed.sh
PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" scripts/ai/verify_fast.sh
```

## Command Results

| Command | Result |
| --- | --- |
| fixed-hash scan | Passed. The stale `6445e4f...` hash now appears only in historical evidence command text, not in current release/migration status wording. |
| `git diff --check` | Passed. |
| `scripts/ai/check_rust_only_runtime.sh` | Passed: `== rust-only-runtime: ok ==`. |
| `scripts/ai/check_cython_removed.sh` | Passed: `== cython-removed: ok ==`. |
| `scripts/ai/verify_fast.sh` | Passed. Rust fmt completed; cargo check and clippy were skipped by current fast-check defaults unless their environment flags are enabled. |

## Behavior Impact

No runtime behavior change. No trading-semantic change.

## Public API Impact

No public API change.

## Rollback Plan

Revert this PR. That restores the stale hard-coded rc.2 commit wording.
