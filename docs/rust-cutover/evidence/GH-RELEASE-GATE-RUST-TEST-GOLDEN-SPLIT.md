# GH Release Gate Rust Test / Golden Trace Split Evidence

Date: 2026-06-13
Executor: Codex

## Task

Release gate optimization after `full-rust-tests` and `full-golden-traces`
remained long-running black-box stages in GitHub Actions.

## Goal

Reduce release gate wall-clock time and improve failure localization by
splitting two large matrix stages into smaller parallel stages:

- `full-rust-tests`
- `full-golden-traces`

## Root Cause

After the dashboard smoke CI fallback fix merged, the release gate progressed
past the previous dashboard failure. Most stages passed, but the remaining
`full-rust-tests` and `full-golden-traces` stages ran for an extended period
without downloadable partial logs from GitHub Actions. This made the release
gate difficult to monitor and slow to diagnose.

## Change Summary

`scripts/ai/verify_full.sh` now supports smaller sub-stages:

- `rust-tests-workspace`
- `rust-tests-common-log-global`
- `rust-tests-live-log-global`
- `rust-tests-live-node-serial`
- `golden-traces-files`
- `golden-traces-harness`
- `golden-traces-market-data`
- `golden-traces-cache-msgbus`
- `golden-traces-backtest`
- `golden-traces-backtest-live-parity`
- `golden-traces-live-sandbox`
- `golden-traces-order-lifecycle`
- `golden-traces-risk-rejection`
- `golden-traces-adapter-payload`

The original aggregate stages remain available:

- `scripts/ai/verify_full.sh rust-tests`
- `scripts/ai/verify_full.sh golden-traces`

`.github/workflows/release-tag.yml` now uses the smaller stages in the release
matrix so GitHub can run them independently and report failures at a narrower
scope.

## Files Changed

- `.github/workflows/release-tag.yml`
- `scripts/ai/verify_full.sh`
- `docs/rust-cutover/evidence/GH-RELEASE-GATE-RUST-TEST-GOLDEN-SPLIT.md`

## Commands Run

```bash
bash -n scripts/ai/verify_full.sh scripts/ai/verify_release.sh scripts/ai/v03_dashboard_smoke.sh
```

Result: passed.

```bash
ruby -e 'require "yaml"; YAML.load_file(".github/workflows/release-tag.yml"); puts "release-tag yaml ok"'
```

Result: passed.

```bash
scripts/ai/verify_full.sh golden-traces-files
```

Result: passed.

```bash
scripts/ai/verify_full.sh golden-traces-harness
```

Result: passed.

```bash
scripts/ai/verify_full.sh rust-tests-live-log-global
```

Result: passed.

```bash
git diff --check
```

Result: passed.

## Behavior Impact

No runtime behavior changed. This only changes release verification stage
granularity.

## Public API Impact

None.

## Migration Note Status

Not required. This is a CI/release verification workflow change.

## Rollback Plan

Revert this PR. The release gate will return to the previous two large
`full-rust-tests` and `full-golden-traces` matrix stages.
