# Backend Freeze GitHub Intake Policy

Date: 2026-07-15
Executor: Codex
Baseline: `ntpro-rust-only-v0.32.0`
Status: ACTIVE

## Decision

Every post-freeze issue and PR must classify itself before implementation. The
classification is durable GitHub state and must agree with the task file,
changed paths, validation evidence, and backend-freeze guard result.

Plain Chinese summary: 后续 issue 和 PR 不能只写“继续开发”。必须明确它是保留基线的
治理、v0.33+ 单独立项，还是后端冻结例外；还必须说明是否触碰冻结字段、是否请求被
禁止的生产能力、如何回滚和如何审计。缺失声明时 fail closed，不进入合并。

## Required Classifications

- `baseline-preserving-governance`: documentation, verification, artifact
  hygiene, intake, or product work that preserves all registered boundaries;
- `v33-separately-scoped`: a new v0.33+ module with its own owner, contract,
  dependencies, rollback, telemetry, and no inherited backend authority;
- `backend-freeze-exception`: a proven frozen-baseline defect or an explicitly
  authorized request to touch a frozen boundary.

## Required Labels

- `backend-freeze`: post-v0.32.0 backend-freeze governance applies;
- `backend-freeze-exception`: explicit exception review is required;
- `v32-baseline`: the issue or PR binds to v0.32.0 baseline facts;
- `v33-separately-scoped`: the work is a separate v0.33+ scope;
- `trading-control-forbidden`: default trading/control capability remains
  forbidden;
- `rust-cutover`: repository Rust-cutover task classification;
- `agent-ready`: the task contract is complete enough for execution.

The last two labels already appeared in the issue form before BFG-005 but were
missing from live GitHub state. BFG-005 creates them as required form
dependencies, in addition to the five backend-freeze labels.

## Required Issue Evidence

The issue form requires:

1. one of the three classifications;
2. an explicit yes/no frozen-boundary touch declaration;
3. a forbidden-capability category, with `none` as the baseline-preserving
   choice;
4. analysis naming affected registry fields and paths;
5. exception authorization, or `Not applicable`;
6. v0.33+ owner/scope/contract/rollback/telemetry evidence, or `Not applicable`;
7. confirmations that no capability is inherited from v0.32.0.

## Required PR Evidence

The PR template repeats the classification and requires the backend-freeze
baseline gate. A PR cannot rely on an issue-only declaration because changed
paths and implementation evidence are available only at PR review time.

For an exception, the PR must identify the authorizing owner, affected frozen
facts, impact, rollback, audit reconstruction, and why an errata or
baseline-preserving fix is insufficient.

## Fail-Closed Routing

- Missing or contradictory declarations block review.
- A non-`none` forbidden capability with no exception classification blocks
  implementation.
- A v0.33+ issue without separate owner/scope/contract/rollback/telemetry proof
  blocks implementation.
- A failed backend-freeze guard blocks merge unless a valid exception proves
  the frozen baseline itself is invalid.
- Labels help routing but never replace issue, PR, task, and evidence content.
