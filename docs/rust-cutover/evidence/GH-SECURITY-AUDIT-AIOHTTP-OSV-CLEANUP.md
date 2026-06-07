# GH Security Audit aiohttp OSV Cleanup Evidence

Date: 2026-06-07
Executor: Codex
Branch: `codex/fix-aiohttp-osv-vulnerabilities`

## Task

Fix the current GitHub `security-audit / osv-scanner` failure before resuming
V03 work.

## Failure Summary

GitHub Actions run `27098438544` failed in the `security-audit` workflow after
the egress allowlist fixes allowed the audit jobs to reach the real scanner
stage.

The failed `osv-scanner` job reported:

- `GHSA-hg6j-4rv6-33pg`, PyPI `aiohttp` `3.13.5`, fixed in `3.14.0`.
- `GHSA-jg22-mg44-37j8`, PyPI `aiohttp` `3.13.5`, fixed in `3.14.0`.

The scanner found both entries from `uv.lock`.

## Change Summary

- Updated the local helper tooling test dependency pin in `pyproject.toml` from
  `aiohttp==3.13.5,<4.0.0` to `aiohttp==3.14.0,<4.0.0`.
- Regenerated `uv.lock` with `uv 0.11.14`.
- Synchronized `[tool.uv].no-build-package` with the regenerated `uv.lock`.

`uv.lock` also normalized to the current root `pyproject.toml` dependency-group
model. This removed stale lock entries left over from the old Python package
surface; those packages are no longer declared in the current helper-tool
dependency groups.

## Behavior Impact

No Rust runtime behavior changes.

No trading semantics change.

No Python product surface is reintroduced. The root `pyproject.toml` remains a
local helper-tool configuration file and does not contain a root `[project]`
package surface.

## Public API Impact

None.

## Migration Note

Not required. This is a CI/security-audit dependency-lock cleanup for local
helper tooling only.

## Validation

Commands run:

```bash
uv self update 0.11.14
uv lock --upgrade-package aiohttp
uv lock --check
uv export --no-hashes --frozen --group test | rg '^aiohttp=='
rg -n "aiohttp==3\\.13\\.5|version = \"3\\.13\\.5\"|aiohttp-3\\.13\\.5" pyproject.toml uv.lock
scripts/check-no-build-packages.sh
scripts/ai/check_rust_only_runtime.sh
scripts/ai/verify_fast.sh
git diff --check
```

Results:

- `uv lock --check` passed.
- Test group export resolves `aiohttp==3.14.0`.
- Residual search found no `aiohttp 3.13.5` lock or pin.
- `scripts/check-no-build-packages.sh` passed with 84 packages in sync.
- `scripts/ai/check_rust_only_runtime.sh` passed.
- `scripts/ai/verify_fast.sh` passed.
- `git diff --check` passed.

## Rollback Plan

Revert this PR to restore the previous `pyproject.toml` and `uv.lock`. That
would also restore the OSV findings, so rollback should only be used if a newer
replacement fix is prepared in the same change window.
