# GH-SECURITY-AUDIT-CHECKOUT-EGRESS-CLEANUP Evidence

Date: 2026-06-07
Executor: Codex
Branch: `codex/fix-security-audit-checkout-egress`

## Task

修复 GitHub Actions 上 `security-audit` push workflow 的 checkout 失败。

## Failure Summary

GitHub run `27097961749` failed in workflow `security-audit`.

- Failed job: `changes`
- Failed step: `Checkout repository`
- Log summary:
  - `fatal: unable to access 'https://github.com/atxinbao/NTPRO/': Failed to connect to github.com port 443`
  - The failure repeated across checkout retries.
- Run URL: `https://github.com/atxinbao/NTPRO/actions/runs/27097961749`

The workflow starts `step-security/harden-runner` before `actions/checkout`.
With egress policy set to `block`, checkout needs `github.com:443` allowed.
The workflow relied on repository variables for common endpoints, but the run
showed GitHub checkout traffic was still blocked.

## Goal

- Let `security-audit` checkout the repository before running audit gating.
- Keep harden-runner enabled.
- Do not weaken or remove the audit jobs.
- Do not change Rust code or runtime behavior.

## Files Changed

- `.github/workflows/security-audit.yml`
- `docs/rust-cutover/evidence/GH-SECURITY-AUDIT-CHECKOUT-EGRESS-CLEANUP.md`

## Change Summary

- Added explicit `github.com:443` to every `security-audit` harden-runner
  allowlist.
- Added explicit `api.github.com:443` for GitHub API operations used by audit
  tooling and SARIF upload paths.
- Kept existing `COMMON_ALLOWED_ENDPOINTS` and `SECURITY_AUDIT_ALLOWED_ENDPOINTS`
  variables intact.

## Commands Run

```bash
gh run view 27097961749 --json name,workflowName,conclusion,status,url,event,headBranch,headSha,jobs
```

Result: failed run confirmed. The failing job was `changes`.

```bash
gh run view 27097961749 --log-failed
```

Result: failed checkout logs showed blocked access to `github.com:443`.

```bash
ruby -e 'require "psych"; ARGV.each { |f| Psych.load_file(f); puts "OK #{f}" }' .github/workflows/security-audit.yml
```

Result: passed. The touched workflow YAML parsed successfully.

```bash
git diff --check
```

Result: passed.

```bash
scripts/ai/verify_fast.sh
```

Result: passed. Toolchain smoke and `cargo fmt --check` passed. The script
reported that workspace cargo check and clippy are outside default fast-smoke
mode.

## Behavior Impact

No Rust runtime behavior changed.

GitHub Actions impact:

- `security-audit` can reach GitHub for checkout under harden-runner egress
  blocking.
- Audit job routing remains unchanged.
- Audit jobs still run only when the workflow gate determines they are needed.

## Public API Impact

None.

## Migration Note

No user-facing migration note is required. This is CI workflow cleanup only.

## Rollback Plan

Revert this PR to restore the previous harden-runner allowlist. If reverting,
`security-audit` may fail again whenever repository variables do not explicitly
allow GitHub checkout traffic.
