# GH verify-release linker crash remediation

Date: 2026-06-13
Executor: Codex

## Local task

- Local task name: GH verify-release linker crash remediation
- Scope: fix and optimize the GitHub `Rust Cutover Release Gate` workflow after
  hosted-runner linker failures and a stuck monolithic release-gate job

## Goal

Keep the release-tag workflow stable and debuggable on GitHub-hosted runners by
splitting the release gate into independent stages, using Rust build cache, and
forcing the hosted runner to use `lld` with serialized Cargo builds. The gate
must fail by stage instead of hanging for a long monolithic `verify_release.sh`
run.

## Files changed

- `.github/workflows/release-tag.yml`
- `scripts/ai/verify_full.sh`
- `scripts/ai/verify_release.sh`
- `docs/rust-cutover/evidence/GH-VERIFY-RELEASE-LINKER-CRASH.md`

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
test binaries at the same time and exhaust the GitHub-hosted runner/linker.

After PR `#260` set `CARGO_BUILD_JOBS=1`, a fresh `workflow_dispatch` run still
became operationally unhealthy:

- Workflow run: `27423501016`
- Head SHA: `5bc497e6e7aa93d615e2d3580c61757de9eb7fbe`
- Observed state: still `in_progress` after more than 90 minutes
- Web log symptom reported by the owner: `error: linking with cc failed: exit status: 1`

GitHub did not expose complete logs while the job was in progress. The run was
cancelled and treated as evidence that one monolithic release job remains too
heavy and too opaque even with serialized Cargo builds.

After all `V031-*` tasks were completed, another hosted release-gate rerun
confirmed the same operational issue:

- Workflow run: `27443173938`
- Head SHA: `335e2a03c31803f1b5d57a306171788d7a3f1613`
- Observed state: still `in_progress` after more than 30 minutes in the single
  `Verify release gate` step
- Action taken: cancelled the stale monolithic run before updating this PR

That run did not produce a hosted PASS and cannot be used as release evidence.
It does prove that the release-gate fix must land before the next v0.3.x tag or
GitHub Release decision.

## Change summary

Updated `.github/workflows/release-tag.yml`:

- keep `CARGO_BUILD_JOBS=1` for serialized per-job linking;
- keep Rust build cache enabled;
- set `CARGO_INCREMENTAL=0` for deterministic CI builds;
- set `RUSTFLAGS=-C link-arg=-fuse-ld=lld` and install `lld` when missing;
- replace the single long `verify-release` job with a staged matrix:
  - `full-fast`;
  - `full-clippy`;
  - `full-rust-tests`;
  - `full-golden-traces`;
  - `full-rust-docs`;
  - `release-build-product-surface`;
  - `release-rust-only-gates`;
  - `release-v02-supervisor-smoke`;
  - `release-v03-supervisor-control-smoke`;
  - `release-v03-dashboard-smoke`;
- keep a final `verify-release` summary job so GitHub still has a stable
  release-gate status check.

Updated `scripts/ai/verify_full.sh`:

- added stage arguments: `fast`, `clippy`, `rust-tests`, `golden-traces`,
  `rust-docs`, and `all`;
- kept no-argument behavior equivalent to the previous full check.

Updated `scripts/ai/verify_release.sh`:

- added stage arguments: `full`, `release-build-product-surface`,
  `rust-only-gates`, `v02-supervisor-smoke`, `v03-supervisor-control-smoke`,
  `v03-dashboard-smoke`, and `all`;
- kept no-argument behavior equivalent to the previous full release check.
- narrowed the release product build from `cargo build --workspace --release`
  to the actual release binaries: `nautilus` and `ntpro-node` from the
  `nautilus-cli` package.
- passed the release binary paths explicitly into the v0.3 supervisor-control
  and dashboard smoke stages.

## Commands run

```bash
gh run list --workflow "Rust Cutover Release Gate" --limit 10
gh run view 27384342541 --json status,conclusion,name,workflowName,event,headBranch,headSha,url,jobs
gh run view 27384342541 --log-failed
gh run view 27421121134 --json status,conclusion,name,workflowName,event,headBranch,headSha,url,jobs
gh run view 27421121134 --log-failed
gh run view 27423501016 --json status,conclusion,name,workflowName,event,headBranch,headSha,url,jobs
gh run cancel 27423501016
gh run view 27443173938 --json status,conclusion,url,createdAt,updatedAt,jobs
gh run cancel 27443173938
rg -n "Rust Cutover Release Gate|verify-release|release gate" .github/workflows scripts/ai docs
bash -n scripts/ai/verify_full.sh scripts/ai/verify_release.sh scripts/ai/v03_supervisor_control_smoke.sh scripts/ai/v03_dashboard_smoke.sh
scripts/ai/verify_full.sh fast
scripts/ai/verify_release.sh rust-only-gates
scripts/ai/verify_release.sh release-build-product-surface
scripts/ai/verify_release.sh v03-supervisor-control-smoke
scripts/ai/verify_release.sh v03-dashboard-smoke
git diff --check
scripts/ai/verify_fast.sh
CARGO_BUILD_JOBS=2 cargo test -p nautilus-architect-ax --tests --no-run
CARGO_BUILD_JOBS=1 cargo test -p nautilus-event-store --tests --no-run
```

## Validation summary

- `bash -n scripts/ai/verify_full.sh scripts/ai/verify_release.sh scripts/ai/v03_supervisor_control_smoke.sh scripts/ai/v03_dashboard_smoke.sh` PASS
- `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/release-tag.yml"); puts "yaml=ok"'` PASS
- `scripts/ai/verify_full.sh fast` PASS
- `scripts/ai/verify_release.sh rust-only-gates` PASS
- `scripts/ai/verify_release.sh release-build-product-surface` PASS; targeted
  release CLI build completed in 4m 03s and validated `nautilus` /
  `ntpro-node` help output.
- `scripts/ai/verify_release.sh v03-supervisor-control-smoke` PASS using
  `target/release/nautilus` and `target/release/ntpro-node`.
- `scripts/ai/verify_release.sh v03-dashboard-smoke` PASS using
  `target/release/nautilus` and `target/release/ntpro-node`.
- `git diff --check` PASS
- `scripts/ai/verify_fast.sh` PASS through `scripts/ai/verify_full.sh fast`
- `CARGO_BUILD_JOBS=2 cargo test -p nautilus-architect-ax --tests --no-run` PASS
- `CARGO_BUILD_JOBS=1 cargo test -p nautilus-event-store --tests --no-run` PASS
- GitHub run `27421121134` FAIL with the same linker `Bus error` class at
  `CARGO_BUILD_JOBS=2`, proving that the release workflow needs
  `CARGO_BUILD_JOBS=1`.
- GitHub run `27423501016` became too long and opaque after
  `CARGO_BUILD_JOBS=1`; the owner observed a linker error in the web log and
  requested release gate optimization instead of further waiting.

## Behavior impact

No runtime behavior change. No trading semantic change. No product-surface
change. This only changes GitHub release-gate execution stability, runtime, and
failure diagnostics on hosted runners.

## Public API impact

None.

## Migration note status

Not required. No public API or runtime contract changed.

## Rollback plan

Revert this PR to restore the previous monolithic release-tag workflow behavior.
If a single stage is noisy, keep the staged workflow and revert only that stage
command or timeout as a smaller rollback.
