# DEXG-007 Docs Build And Link Gate

Date: 2026-07-15
Executor: Codex
GitHub issue: #1086
Milestone: post-backend-docs-examples-governance
Status: READY FOR PR

## Goal

Make the supported Rust docs and examples surface deterministically buildable
and locally link-checkable after legacy cleanup.

Plain Chinese summary: 本任务把 `make docs` 收敛为 Rust docs build 加本地
docs/examples governance gate；内部链接检查不依赖 lychee 或 Python 第三方包，网络
外链检查保留为独立 periodic target。GitHub smoke 对每个 PR 运行相同 gate。

## Dependency

DEXG-002, DEXG-004, DEXG-005, and DEXG-006 are merged and closed.

## Scope

Included:

- add the deterministic docs/examples governance script;
- validate current public docs plus governance/migration local links and images;
- validate integration/concept authority, tutorial assets, Rust examples, and
  backend freeze;
- remove retired Python URL exclusions from the external link target;
- run the gate in GitHub smoke;
- execute the actual Rust docs build, then clean generated `target/` output.

Not included:

- changing runtime or adapter behavior;
- requiring network access for the merge-blocking local link gate;
- editing frozen v0.32.0 release files.

## Acceptance Criteria

- `make docs-check-links` passes without legacy exclusions;
- `make docs` builds Rust crate docs and passes the local gate;
- the external lychee target contains no retired Python exclusions;
- GitHub smoke runs the same gate;
- generated target output is cleaned after local validation.

## Validation

```bash
bash -n scripts/ai/check_docs_examples_governance.sh
scripts/ai/check_docs_examples_governance.sh
make docs-check-links
make docs
cargo clean
scripts/ai/verify_fast.sh
git diff --check
```
