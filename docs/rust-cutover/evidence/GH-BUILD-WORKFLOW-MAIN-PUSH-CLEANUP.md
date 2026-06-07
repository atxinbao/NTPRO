# GH-BUILD-WORKFLOW-MAIN-PUSH-CLEANUP

Date: 2026-06-07
Executor: Codex
Branch: codex/fix-build-workflow-github-hosted-ci

## Task

Fix the remaining GitHub Actions errors before resuming V03 development.

## Problem

After the security-audit fixes, the latest main push still started the upstream-heavy
`build` and `build-v2` workflows:

- `build` run `27098985722`
- `build-v2` run `27098985712`

Those runs failed before repository checkout because harden-runner blocked
`github.com`, and their push pre-commit jobs also targeted upstream self-hosted
runner labels that are not available in the standalone NTPRO repository.

## Change

- Removed `main` from `.github/workflows/build.yml` push branches.
- Added `workflow_dispatch` to `.github/workflows/build-v2.yml`.
- Removed `main` from `.github/workflows/build-v2.yml` push branches.
- Kept both workflows available for explicit test branches and manual runs.

The regular main/PR safety path remains covered by:

- `Rust Cutover Smoke`
- `security-audit`
- targeted local validation recorded by each task evidence file

## Files changed

- `.github/workflows/build.yml`
- `.github/workflows/build-v2.yml`
- `docs/rust-cutover/evidence/GH-BUILD-WORKFLOW-MAIN-PUSH-CLEANUP.md`

## Behavior impact

No runtime behavior change. This is a CI trigger cleanup only.

## Public API impact

None.

## Migration note

Not required. Developers can still run the upstream-heavy workflows manually or
from explicit CI test branches.

## Validation

- `git diff --check`: passed.
- `ruby -e 'require "yaml"; ARGV.each { |path| YAML.load_file(path); puts "OK #{path}" }' .github/workflows/build.yml .github/workflows/build-v2.yml`: passed.
- `rg -n "branches: \[main|^[[:space:]]+- main$" .github/workflows/build.yml .github/workflows/build-v2.yml`: no matches, as expected.
- `scripts/ai/verify_fast.sh`: passed.

## Rollback plan

Re-add `main` to the workflow push branch list if NTPRO later provisions the
required upstream-compatible runner pools and egress configuration.
