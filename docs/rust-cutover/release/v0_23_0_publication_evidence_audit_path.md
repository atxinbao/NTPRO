# v0.23.0 Publication Evidence Audit Path

Date: 2026-07-04
Executor: Codex
Task: `V231-005`
GitHub issue: `#741`

## Purpose

This document defines the audit path for v0.23.0 publication evidence. The
canonical proof is the source tree plus GitHub remote facts. The local
`release-publication-evidence/ntpro-rust-only-v0.23.0.json` file is a generated
artifact and is not the only source of truth.

Plain Chinese summary: 本文件说明 v0.23.0 publication evidence 的复核方式。
审计者不需要依赖未跟踪的本地 `release-publication-evidence/` 目录；只要有源码树里的
manifest / closeout evidence / 本文件，再读取 GitHub Release、Actions run 和 tag
remote facts，就可以重建发布事实。

## Policy

```text
publication evidence strategy = source_tree_plus_github_remote
tracked audit path = docs/rust-cutover/release/v0_23_0_publication_evidence_audit_path.md
local generated evidence path = release-publication-evidence/ntpro-rust-only-v0.23.0.json
local generated evidence required in source tree = false
remote reconstruction required = true
secret / token / credential / raw sensitive material = forbidden
```

## Reconstructable Facts

```text
release tag = ntpro-rust-only-v0.23.0
release name = NTPRO Rust-only v0.23.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.23.0
GitHub Release draft = false
GitHub Release prerelease = false
published at = 2026-07-03T18:34:39Z
target commitish = main
tag ref = refs/tags/ntpro-rust-only-v0.23.0
tag object type = commit
tag commit = 783b024621116d50feaf418f12cb95fb95f87575
hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28673868094
hosted release gate workflow = Rust Cutover Release Gate
hosted release gate status = completed
hosted release gate conclusion = success
hosted release gate completed at = 2026-07-03T18:29:30Z
hosted release gate headSha = 783b024621116d50feaf418f12cb95fb95f87575
hosted release gate jobs = 66/66 success
release publication after gate = pass
```

## Remote Reconstruction Commands

```text
gh release view ntpro-rust-only-v0.23.0 --repo atxinbao/NTPRO --json tagName,name,isDraft,isPrerelease,url,publishedAt,targetCommitish
gh run view 28673868094 --repo atxinbao/NTPRO --json status,conclusion,workflowName,headSha,url,createdAt,updatedAt,jobs
gh api repos/atxinbao/NTPRO/git/ref/tags/ntpro-rust-only-v0.23.0
```

## Source-Tree Inputs

```text
docs/rust-cutover/release/v0_23_0_release_manifest.json
docs/rust-cutover/release/v0_23_0_release_closeout_evidence.md
docs/rust-cutover/release/v0_23_0_readiness_report.md
docs/rust-cutover/evidence/V230-007.md
docs/rust-cutover/evidence/V231-005.md
```

## Boundary

```text
publication evidence audit only = true
release permission model changed = false
temporary target artifacts tracked = false
local generated publication evidence sole source of truth = false
new_submit_capability = false
production_order_mutation_allowed = false
dashboard_operation_controls_enabled = false
product_grade_live_trading_terminal_claim = false
```
