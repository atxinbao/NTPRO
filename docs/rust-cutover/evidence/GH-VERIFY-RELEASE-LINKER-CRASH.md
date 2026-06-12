# GH verify-release linker crash remediation

Date: 2026-06-12
Executor: Codex

## Local task

- Local task name: GH verify-release linker crash remediation
- Scope: fix the GitHub `Rust Cutover Release Gate / verify-release (push)` workflow failure on tag push

## Goal

Keep the release-tag workflow stable on GitHub-hosted runners by reducing Cargo
parallelism and reusing Rust build cache, so the release gate does not fail late
with linker crashes during full workspace/test builds.

## Files changed

- `.github/workflows/release-tag.yml`

## Root cause summary

The original failing GitHub Actions run was:

- Workflow: `Rust Cutover Release Gate`
- Event: `push`
- Ref: `ntpro-rust-only-v0.3.0`
- Run: `27384342541`

The failure was not caused by a Rust source regression. The failing step was
`Verify release gate`, and the log showed the linker crashing while compiling
`nautilus-architect-ax` targets:

- `collect2: fatal error: ld terminated with signal 7 [Bus error], core dumped`
- `could not compile nautilus-architect-ax (bin "ax-flatten")`
- `could not compile nautilus-architect-ax (bin "ax-ws-orders")`
- `could not compile nautilus-architect-ax (test "websocket")`

The release-tag workflow did not set any Cargo job limit and also disabled the
Rust cache, so the GitHub-hosted runner hit a heavy cold full build and crashed
inside the linker.

After PR `#258` set `CARGO_BUILD_JOBS=2` and enabled cache, a fresh
`workflow_dispatch` run on main after PR `#259` still failed with the same
class of hosted-runner linker crash:

- Workflow run: `27421121134`
- Head SHA: `afc805396ad731e93f99252fbf3ca9e81010753a`
- Failing targets: `nautilus-event-store` tests `verifier` and `writer`
- Error: `collect2: fatal error: ld terminated with signal 7 [Bus error], core dumped`

That second failure showed that two concurrent Cargo jobs can still link large
test binaries at the same time and exhaust the GitHub-hosted runner/linker. The
release gate therefore needs serialized Cargo builds on hosted runners.

## Change summary

Updated `.github/workflows/release-tag.yml`:

- set `CARGO_BUILD_JOBS=1` at the job level for serialized release-gate linking;
- changed `setup-rust-toolchain` from `cache: false` to `cache: true`.

## Commands run

```bash
gh run list --workflow "Rust Cutover Release Gate" --limit 10
gh run view 27384342541 --json status,conclusion,name,workflowName,event,headBranch,headSha,url,jobs
gh run view 27384342541 --log-failed
gh run view 27421121134 --json status,conclusion,name,workflowName,event,headBranch,headSha,url,jobs
gh run view 27421121134 --log-failed
rg -n "Rust Cutover Release Gate|verify-release|release gate" .github/workflows scripts/ai docs
git diff --check
scripts/ai/verify_fast.sh
CARGO_BUILD_JOBS=2 cargo test -p nautilus-architect-ax --tests --no-run
CARGO_BUILD_JOBS=1 cargo test -p nautilus-event-store --tests --no-run
```

## Validation summary

- `git diff --check` PASS
- `scripts/ai/verify_fast.sh` PASS
- `CARGO_BUILD_JOBS=2 cargo test -p nautilus-architect-ax --tests --no-run` PASS
- `CARGO_BUILD_JOBS=1 cargo test -p nautilus-event-store --tests --no-run` PASS
- GitHub run `27421121134` FAIL with the same linker `Bus error` class at
  `CARGO_BUILD_JOBS=2`, proving that the release workflow needs
  `CARGO_BUILD_JOBS=1`.

## Behavior impact

No runtime behavior change. No trading semantic change. No product-surface
change. This only changes GitHub release-gate execution stability and rebuild
cost on hosted runners.

## Public API impact

None.

## Migration note status

Not required. No public API or runtime contract changed.

## Rollback plan

Revert this PR to restore the previous release-tag workflow behavior. If the
new cache path causes operational trouble, keep `CARGO_BUILD_JOBS=1` and revert
only the cache toggle as a smaller rollback.
