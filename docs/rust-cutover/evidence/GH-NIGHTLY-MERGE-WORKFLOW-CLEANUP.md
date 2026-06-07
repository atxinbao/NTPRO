# GH-NIGHTLY-MERGE-WORKFLOW-CLEANUP Evidence

Date: 2026-06-07
Executor: Codex
Branch: `codex/fix-nightly-merge-workflow`

## Task

修复 GitHub Actions 上 `nightly-merge` 定时任务失败的问题。

## Failure Summary

GitHub run `27096695782` failed in workflow `nightly-merge`.

- Failed job: `check-develop-status`
- Failed step: `Checkout repository`
- Log summary: `Input required and not supplied: token`
- Run URL: `https://github.com/atxinbao/NTPRO/actions/runs/27096695782`

The workflow was still an upstream NautilusTrader develop-to-nightly merge job.
It expected:

- `secrets.NIGHTLY_TOKEN`
- remote `develop`
- remote `nightly`
- upstream `nautechsystems/nautilus_trader` workflow status

Current NTPRO remote only has `main`, so the scheduled workflow is not a valid
NTPRO release path.

## Goal

- Stop the obsolete scheduled `nightly-merge` failure.
- Avoid restoring upstream develop/nightly automation.
- Keep a manual, self-documenting no-op workflow so GitHub history remains
  understandable.

## Files Changed

- `.github/workflows/nightly-merge.yml`
- `docs/rust-cutover/evidence/GH-NIGHTLY-MERGE-WORKFLOW-CLEANUP.md`

## Change Summary

- Removed the scheduled trigger.
- Removed `NIGHTLY_TOKEN` usage.
- Removed upstream `develop` workflow polling.
- Removed local `nightly` branch merge/push logic.
- Replaced the workflow with a `workflow_dispatch` no-op explaining that NTPRO
  uses `main` and should create a new Rust-only nightly workflow if needed.

## Commands Run

```bash
gh run view 27096695782 --json name,workflowName,conclusion,status,url,event,headBranch,headSha,jobs
```

Result: failed run confirmed. The failing job was `check-develop-status`.

```bash
gh run view 27096695782 --log-failed
```

Result: failed step log showed `Input required and not supplied: token`.

```bash
git ls-remote --heads origin main develop nightly nightly-merge-test
```

Result: only `refs/heads/main` exists. No NTPRO remote `develop` or `nightly`
branch exists.

## Behavior Impact

No Rust runtime behavior changed.

GitHub Actions impact:

- The obsolete scheduled nightly merge no longer runs daily.
- Manual dispatch is still available and returns a clear no-op explanation.
- No branch merge, tag, release, or publishing behavior is added.

## Public API Impact

None.

## Migration Note

No user-facing migration note is required. Release operators should not use the
old upstream develop-to-nightly merge model for NTPRO.

## Rollback Plan

Revert this PR to restore the previous scheduled workflow. If reverting, also
create the required `NIGHTLY_TOKEN`, `develop`, and `nightly` setup or the
workflow will fail again.
