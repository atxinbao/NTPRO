# GitHub Issue Register

Date: 2026-06-05
Executor: Codex

## Purpose

This register mirrors GitHub audit issues into local agentflow records so that
NTPRO planning can continue from repository-owned task state instead of relying
on transient GitHub UI state or chat context.

At registration time, GitHub has no open issues and no open pull requests.

## Closed GitHub Audit Issues

| Local task | GitHub issue | Status | Scope | Evidence |
|------------|--------------|--------|-------|----------|
| `GH-155` | [#155](https://github.com/atxinbao/NTPRO/issues/155) | closed | Implement real `config validate` CLI path. | `docs/rust-cutover/evidence/GH-155-CONFIG-VALIDATE-CLI.md` |
| `GH-156` | [#156](https://github.com/atxinbao/NTPRO/issues/156) | closed | Implement real `data inspect` / `data validate` CLI path. | `docs/rust-cutover/evidence/GH-156-DATA-INSPECT-VALIDATE-CLI.md` |
| `GH-157` | [#157](https://github.com/atxinbao/NTPRO/issues/157) | closed | Align CLI help text for unimplemented commands. | `docs/rust-cutover/evidence/GH-157-CLI-HELP-CONTRACT.md` |
| `GH-158` | [#158](https://github.com/atxinbao/NTPRO/issues/158) | closed | Replace product-reachable `todo!()` panics with explicit errors. | `docs/rust-cutover/evidence/GH-158-TODO-PANIC-EXPLICIT-ERRORS.md` |
| `GH-159` | [#159](https://github.com/atxinbao/NTPRO/issues/159) | closed | Harden live node startup stop/shutdown responsiveness. | `docs/rust-cutover/evidence/GH-159-LIVE-STARTUP-CANCELLATION.md` |
| `GH-160` | [#160](https://github.com/atxinbao/NTPRO/issues/160) | closed | Create ignored tests risk register. | `docs/rust-cutover/evidence/GH-160-IGNORED-TESTS-RISK-REGISTER.md` |
| `GH-161` | [#161](https://github.com/atxinbao/NTPRO/issues/161) | closed | Clarify `verify_fast.sh` scope and release verification boundary. | `docs/rust-cutover/evidence/GH-161-VERIFY-FAST-BOUNDARY.md` |

## Local Follow-up Audit Backlog

The following issues are recorded locally first. They should be promoted to
GitHub issues only when the owner decides to execute or externally track them.

| Local task | Status | Risk | Summary |
|------------|--------|------|---------|
| `NQA-001` | TODO | medium | Close the nine v0.2 `QA_REQUIRED` tasks and produce a readiness report. |
| `NAUDIT-001` | TODO | critical | Clean up root Python package metadata and strengthen the Rust-only gate. |
| `NAUDIT-002` | TODO | medium | Lock the CLI capability matrix and fix misleading stub output. |
| `NAUDIT-003` | TODO | medium | Unignore passing production-bug cache tests. |
| `NAUDIT-004` | TODO | high | Replace product-reachable runtime panics with explicit errors or rejection paths. |
| `NAUDIT-005` | TODO | medium | Classify PostgreSQL cache adapter support status. |
| `NAUDIT-006` | TODO | high | Add live adapter cancellation contract and mock cancellation evidence. |
| `NAUDIT-007` | TODO | medium | Create unsafe/plugin audit register before plugin productization. |

## Operating Rule

`GH-*` tasks in this register are historical mirrors and should not be
redispatched. `NAUDIT-*` and `NQA-*` tasks are executable only after an explicit
owner instruction selects the next task.
