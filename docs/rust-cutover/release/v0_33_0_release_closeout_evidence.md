# v0.33.0 Release Closeout Contract

Date: 2026-07-21
Executor: Codex
Release: `ntpro-rust-only-v0.33.0`
Status: SOURCE-CONTROLLED CLOSEOUT CONTRACT

## Summary

This file defines the evidence that must be reconstructed after v0.33.0 is
published. It intentionally does not pre-record a future tag SHA, hosted run
ID, publication timestamp, or closed milestone state.

中文摘要：本文件固定发布后的核验规则，不提前伪造尚未发生的 SHA、run id 或发布时间。
真实 tag、release、gate、issue、PR 和 milestone 状态必须从 GitHub live state 重建。

## Required Reconstruction

```text
release tag = ntpro-rust-only-v0.33.0
release name = NTPRO Rust-only v0.33.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.33.0
release body = docs/rust-cutover/release/v0_33_0_release_notes.md
release manifest = docs/rust-cutover/release/v0_33_0_release_manifest.json
hosted release gate status = completed
hosted release gate conclusion = success
hosted release gate head SHA = release tag peeled commit
release publication time >= hosted release gate completion time
exact issue set = #1120-#1126
exact PR set = #1134-#1140
all exact issues = closed
all exact PRs = merged
open repository issues after closeout = 0
open repository PRs after closeout = 0
milestone v0.33.0-backend-maintenance must close after release publication = true
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
generated publication evidence sole proof allowed = false
```

## Rollback

Reject the candidate before publication when any gate fails. After publication,
revert code on `main` and create a separately scoped corrective release when
needed. Published tags are not rewritten, and v0.32.0 frozen evidence is never
modified as part of v0.33.0 rollback.

## Boundary

v0.33.0 remains maintenance-only. It inherits no backend go-live, production
submit/mutation, adapter call/send, live exchange request, retry scheduler,
automatic remediation/recovery, Dashboard/Admin/Trader Terminal trading
controls, frontend completion, or product-grade live terminal authority.
