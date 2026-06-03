# RC2-TAG-DOCS Evidence

Date: 2026-06-04
Executor: Codex
Task ID: RC2-TAG-DOCS

## Plain Summary

This PR prepares the repository documents for the approved
`ntpro-rust-only-rc.2` tag.

In plain terms: `ntpro-rust-only-rc.1` still points to the commit before the
README cleanup and Python test removal. The human owner approved creating
`ntpro-rust-only-rc.2` so the release-candidate source point includes PR #122.
This PR updates the public documents first, so the source package for rc.2 does
not incorrectly call itself rc.1.

## Scope

- Update README current milestone from `ntpro-rust-only-rc.1` to
  `ntpro-rust-only-rc.2`.
- Update release documents to record rc.1 as the pre-cleanup tag and rc.2 as
  the current post-cleanup tag-only release candidate.
- Update migration/status notes that describe the current tag-only release
  candidate.
- Do not create a tag in this PR.
- Do not publish a GitHub Release.
- Do not modify Rust runtime or trading code.

## Files Changed

- `README.md`
- `docs/rust-cutover/release/*.md`
- `docs/rust-cutover/migration/*.md`

## Commands Run

```bash
git ls-remote --tags origin refs/tags/ntpro-rust-only-rc.2 'refs/tags/ntpro-rust-only-rc.2^{}'
rg -n "ntpro-rust-only-rc\\.1|a886e2ac|ntpro-rust-only-rc\\.2|6445e4f" README.md docs/rust-cutover/release docs/rust-cutover/migration docs/rust-cutover/evidence/RC-CLEANUP-001.md
git diff --check
PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" scripts/ai/check_rust_only_runtime.sh
PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" scripts/ai/check_cython_removed.sh
PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" scripts/ai/verify_fast.sh
```

## Command Results

| Command | Result |
| --- | --- |
| `git ls-remote --tags origin refs/tags/ntpro-rust-only-rc.2 'refs/tags/ntpro-rust-only-rc.2^{}'` | Passed. No remote rc.2 tag exists before this PR/tag-prep work. |
| rc1/rc2 reference scan | Passed. Current public milestone points to `ntpro-rust-only-rc.2`; `ntpro-rust-only-rc.1` remains only as historical pre-cleanup tag evidence. |
| `git diff --check` | Passed. |
| `scripts/ai/check_rust_only_runtime.sh` | Passed: `== rust-only-runtime: ok ==`. |
| `scripts/ai/check_cython_removed.sh` | Passed: `== cython-removed: ok ==`. |
| `scripts/ai/verify_fast.sh` | Passed. Rust fmt completed; cargo check and clippy were skipped by current fast-check defaults unless their environment flags are enabled. |

## Behavior Impact

No runtime behavior change. No trading-semantic change.

## Public API Impact

No public API change.

## Migration Note Status

Updated. Documents now distinguish rc.1 as the pre-cleanup tag-only candidate
and rc.2 as the post-cleanup tag-only candidate.

## Rollback Plan

Revert this PR. That restores the previous rc.1-only release documentation.
