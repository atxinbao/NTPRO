# GH-SECURITY-AUDIT-TOOL-EGRESS-CLEANUP Evidence

Date: 2026-06-07
Executor: Codex
Branch: `codex/fix-security-audit-tool-egress`

## Task

继续修复 `security-audit` workflow 在 harden-runner egress block 下的工具安装
和外部审计 API 访问失败。

## Failure Summary

After PR #218 fixed checkout egress, GitHub run `27098165547` reached the audit
jobs but failed during tool setup and audit API access.

Observed failures:

- `pip-audit` and `zizmor` failed in `Install uv` while fetching
  `https://raw.githubusercontent.com/astral-sh/versions/main/v1/uv.ndjson`.
- `cargo-audit`, `cargo-deny`, and `cargo-vet` failed while installing Rust
  toolchains from `https://static.rust-lang.org/dist/...`.
- `osv-scanner` failed while posting to `https://api.osv.dev/v1/querybatch`.

These failures happened after successful checkout, which proved the previous
checkout egress fix worked but the security-tool endpoints were still blocked.

## Goal

- Keep `step-security/harden-runner` enabled.
- Allow the existing security audit tools to download their pinned toolchains
  and query their audit services.
- Do not disable or skip security audit jobs.
- Do not change Rust code or runtime behavior.

## Files Changed

- `.github/workflows/security-audit.yml`
- `docs/rust-cutover/evidence/GH-SECURITY-AUDIT-TOOL-EGRESS-CLEANUP.md`

## Change Summary

Added explicit harden-runner allowlist entries needed by the existing
`security-audit` jobs:

- GitHub release/action assets:
  - `raw.githubusercontent.com:443`
  - `objects.githubusercontent.com:443`
  - `release-assets.githubusercontent.com:443`
  - `github-releases.githubusercontent.com:443`
- Rust toolchain and Cargo registry:
  - `static.rust-lang.org:443`
  - `crates.io:443`
  - `index.crates.io:443`
  - `static.crates.io:443`
- OSV vulnerability API:
  - `api.osv.dev:443`
- Python package audit tooling:
  - `pypi.org:443`
  - `files.pythonhosted.org:443`

Existing repository variable allowlists remain in place.

## Commands Run

```bash
gh run view 27098165547 --json status,conclusion,url,jobs
```

Result: checkout passed; audit jobs failed later in tool install/API steps.

```bash
gh run view 27098165547 --log-failed
```

Result: logs identified blocked `raw.githubusercontent.com`,
`static.rust-lang.org`, and `api.osv.dev` endpoints.

```bash
ruby -e 'require "psych"; ARGV.each { |f| Psych.load_file(f); puts "OK #{f}" }' .github/workflows/security-audit.yml
```

Result: passed. The touched workflow YAML parsed successfully.

```bash
git diff --check
```

Result: passed.

```bash
python3 - <<'PY'
from pathlib import Path
lines = [line.strip() for line in Path('.github/workflows/security-audit.yml').read_text().splitlines()]
for endpoint in ['raw.githubusercontent.com:443','objects.githubusercontent.com:443','release-assets.githubusercontent.com:443','github-releases.githubusercontent.com:443','static.rust-lang.org:443','crates.io:443','index.crates.io:443','static.crates.io:443','api.osv.dev:443','pypi.org:443','files.pythonhosted.org:443']:
    count = sum(1 for line in lines if line == endpoint)
    print(f'{endpoint} {count}')
    assert count == 7, (endpoint, count)
PY
```

Result: passed. Every required endpoint appears in all 7 harden-runner
allowlists.

```bash
scripts/ai/verify_fast.sh
```

Result: passed. Toolchain smoke and `cargo fmt --check` passed. The script
reported that workspace cargo check and clippy are outside default fast-smoke
mode.

## Behavior Impact

No Rust runtime behavior changed.

GitHub Actions impact:

- `security-audit` keeps egress blocking enabled.
- The audit tools can reach the specific package, toolchain, and vulnerability
  endpoints they already depend on.
- No audit job is removed or converted to continue-on-error.

## Public API Impact

None.

## Migration Note

No user-facing migration note is required. This is CI workflow cleanup only.

## Rollback Plan

Revert this PR to restore the previous endpoint allowlist. If reverting, the
workflow may again pass checkout but fail while installing uv, Rust audit tools,
or querying OSV.
