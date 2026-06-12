# GH Release Dashboard Smoke CI Fallback Evidence

Date: 2026-06-13
Executor: Codex

## Task

Local release-gate remediation for the GitHub Actions failure in
`release-v03-dashboard-smoke`.

## Goal

Make `scripts/ai/v03_dashboard_smoke.sh` runnable in a clean GitHub Actions
runner where the local Codex Playwright wrapper path is unavailable, while
keeping the existing full browser smoke path for local environments that have
the wrapper installed.

## Root Cause

The release gate job reached the v0.3 dashboard smoke after the release binary
build completed, then failed because the GitHub runner did not have:

```text
/home/runner/.codex/skills/playwright/scripts/playwright_cli.sh
```

This was an environment coupling in the smoke script, not a Rust linker failure
or dashboard runtime failure.

## Change Summary

- Kept the existing Playwright wrapper browser smoke path unchanged when the
  wrapper exists.
- Added an API/HTML fallback path when the wrapper is unavailable.
- Added `NTPRO_V03_DASHBOARD_REQUIRE_PLAYWRIGHT=1` as a strict mode for local
  runs that must fail if the Playwright wrapper is missing.
- The fallback validates:
  - dashboard HTML shell loads;
  - dashboard JavaScript includes the expected snapshot/control wiring;
  - `/api/snapshot` exposes both sandbox nodes and expected sections;
  - initial control availability matches the expected sandbox state;
  - reconnect data/execution actions return `not_supported`;
  - pause, resume, start, and stop control actions update node lifecycle state;
  - final node state is `sandbox-a=running`, `sandbox-b=stopped`.

## Files Changed

- `scripts/ai/v03_dashboard_smoke.sh`
- `docs/rust-cutover/evidence/GH-RELEASE-DASHBOARD-SMOKE-CI-FALLBACK.md`

## Commands Run

```bash
bash -n scripts/ai/v03_dashboard_smoke.sh scripts/ai/verify_release.sh
```

Result: passed.

```bash
CODEX_HOME=/tmp/ntpro-no-codex \
  NTPRO_V03_010_SKIP_BUILD=1 \
  NTPRO_V03_NAUTILUS_BIN=/Users/mac/Documents/NTPRO/target/release/nautilus \
  NTPRO_V03_NODE_BIN=/Users/mac/Documents/NTPRO/target/release/ntpro-node \
  scripts/ai/v03_dashboard_smoke.sh
```

Result: passed.

Output summary:

```text
Playwright wrapper unavailable; using API/HTML dashboard smoke fallback
api_html_dashboard_smoke status=ok final_states={'sandbox-a': 'running', 'sandbox-b': 'stopped'}
v03_dashboard_smoke status=ok mode=api-html-fallback ... nodes=sandbox-a,sandbox-b
```

```bash
CODEX_HOME=/tmp/ntpro-no-codex scripts/ai/verify_release.sh v03-dashboard-smoke
```

Result: passed.

Output summary:

```text
Finished `release` profile [optimized] target(s) in 2m 27s
api_html_dashboard_smoke status=ok final_states={'sandbox-a': 'running', 'sandbox-b': 'stopped'}
== verify_release complete ==
```

```bash
scripts/ai/verify_full.sh fast
```

Result: passed.

```bash
git diff --check
```

Result: passed.

## Behavior Impact

No trading runtime behavior changed. The dashboard smoke script now has a
portable CI fallback when the local Codex Playwright wrapper is absent.

## Public API Impact

None.

## Migration Note Status

Not required. This is a release verification script portability fix.

## Rollback Plan

Revert the `scripts/ai/v03_dashboard_smoke.sh` change and this evidence file.
That returns the smoke script to requiring the local Playwright wrapper for all
environments.
