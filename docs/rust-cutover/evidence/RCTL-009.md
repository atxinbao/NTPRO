# RCTL-009 Evidence - PR plain Chinese summary and medium-risk automation policy

- Date: 2026-05-30
- Executor: Codex
- Task ID: RCTL-009
- Owner role: control_scope_agent
- Review role: verification_release_gatekeeper
- Risk level: medium
- Branch: ai/RCTL-009-pr-summary-risk-automation-policy

## 大白话说明

这次改的是“以后 PR 怎么写”和“自动化怎么继续跑”。以后每个 PR 都必须有中文大白话说明，写清楚改了什么、没改什么、验证结果和影响。任务风险统一改成中风险，这样自动化不会再因为之前的 high/critical 风险字段一直停住。另一个实际阻塞也修了：本地 `main` 分叉时，自动调度会从 `origin/main` 起新分支，不再卡在本地 `main` 必须能 fast-forward。

## Files changed

- `AGENTS.md`
- `.github/pull_request_template.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCTL-009.json`
- `docs/rust-cutover/AGENT_ROLES.md`
- `docs/rust-cutover/TASK_EXECUTION.md`
- `docs/rust-cutover/automation/PR_AUTODISPATCH.md`
- `docs/rust-cutover/tasks/RCTL-009.md`
- `docs/rust-cutover/evidence/RCTL-009.md`
- `scripts/control/close_merged_pr.py`
- `scripts/control/dispatch_next.py`

No runtime, trading, adapter, Python, PyO3, Cython, or Cargo code changed.

## Goal

Add a persistent rule that PR content includes a plain Chinese summary, normalize current task risk metadata to medium, and make local automation continue from `origin/main` when local `main` has diverged.

## Behavior impact

- PR bodies now have an explicit `大白话说明` section.
- Agent rules require the plain Chinese summary for every PR, not only high-risk work.
- `.agentflow/state/task_status.json` risk metadata is normalized to `medium`.
- `close_merged_pr.py` no longer fails only because local `main` cannot fast-forward.
- `dispatch_next.py` creates task branches from `origin/main`, so local `main` divergence no longer blocks dispatch.
- Removal, release, and explicit gate task prefixes remain manual-gated by `dispatch_next.py`.

## Public API impact

No public API changed.

## Migration note status

No migration note is required because this is control-plane automation and documentation only.

## Rollback plan

Revert this PR to restore the previous PR template, risk metadata, and local dispatch behavior. No runtime rollback is required.

## Commands run

```bash
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RCTL-009.json >/dev/null
python3 scripts/ai/validate_agentflow_roles.py
git diff --check
jq -r '.tasks | to_entries[] | .value.risk_level' .agentflow/state/task_status.json | sort | uniq -c
env PATH=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc scripts/ai/verify_fast.sh
```

Result: passed. Risk metadata check returned `101 medium`. `verify_fast` completed with cargo check and clippy skipped by fast-mode defaults.
