# Docs And Examples Governance Closeout Contract

Date: 2026-07-15
Executor: Codex
Milestone: `post-backend-docs-examples-governance` (#32)
Status: SOURCE-CONTROLLED CLOSEOUT CONTRACT

## Summary

This contract closes the post-v0.32.0 docs/examples governance work after
DEXG-001 through DEXG-008 merge. Final issue, PR, workflow, main-branch, and
milestone state must be reconstructed from live GitHub after the DEXG-008 PR
merges.

Plain Chinese summary: 本批工作在 v0.32.0 后端基线冻结后清理文档与 examples，
不发布 v0.32.1，也不增加后端能力。DEXG-008 合并后，GitHub live state 必须证明
#1080-#1087 全部 closed、对应 PR 全部 merged、milestone #32 closed，且源码 gate
继续证明 Rust-only 文档入口、examples、内部链接、assets 和后端冻结边界有效。

## Exact Governance Scope

```text
DEXG issue set = #1080-#1087
DEXG issue count = 8
DEXG-001 PR = #1088, merge commit b9acb49204c64b35f287f88d737a3c7d922b0d85
DEXG-002 PR = #1089, merge commit de6b0a69ccef8f0df9041d797447972534c5e348
DEXG-003 PR = #1090, merge commit 3c633abe01fdcf47ba201b7af6e6da22d8942805
DEXG-004 PR = #1091, merge commit f8faaf00a4884913a873728b28e85c6397310eac
DEXG-005 PR = #1092, merge commit 4b3be4971f7cf26ce1fd25d2c6a143c44b217dfc
DEXG-006 PR = #1093, merge commit 55a7ef348b7c49782a2c557858f244c4e9718358
DEXG-007 PR = #1094, merge commit 9ce4f26f81dac39d6612bb40f65b50e648e2344c
DEXG-008 PR = must merge before closeout
```

## Delivered Governance

- classified frozen release, retained audit, canonical examples, rewrite,
  removable, and ephemeral surfaces;
- repaired canonical Rust example paths and added deterministic integrity
  checks;
- retired the legacy Python API appendix, docs-python build entry, and its
  exclusive dependencies;
- retired unsupported Python-first tutorials, how-to, developer guide, and
  newly orphaned media with migration records;
- marked integration and concept documentation with Rust-only authority and
  replaced retired Python API links with tracked Rust sources;
- made stable Rust workspace docs plus deterministic local governance the
  supported `make docs` route;
- added the same docs/examples governance gate to GitHub smoke.

## Required Post-Merge Reconstruction

After the DEXG-008 PR merges, all conditions are required:

```text
origin/main contains all DEXG merge commits = true
issues #1080-#1087 closed = 8/8
PRs #1088 through DEXG-008 PR merged = all
required hosted checks completed/success = all
milestone #32 open issues = 0
milestone #32 closed issues = 8
milestone #32 state = closed
open post-backend-docs-examples-governance PRs = 0
docs/examples governance gate = pass
backend freeze baseline guard = pass
v0.32.0 frozen release files changed = false
backend patch scheduled = false
```

The DEXG-008 PR cannot contain its own final merge SHA or post-merge workflow
conclusion without creating circular proof. Those facts are intentionally bound
through `source_tree_plus_github_remote`, then recorded in the final GitHub
issue and milestone closeout comments.

## Generated Artifact State

The supported source tree does not require local generated outputs as audit
authority. At pre-merge validation:

```text
target/ = absent after cargo clean
release-publication-evidence/ = absent
docs/examples .DS_Store = absent
repository __pycache__ outside .venv = absent
```

The `.venv` environment may contain dependency-owned Python caches and remains
outside source cleanup scope. Future docs or validation commands may recreate
ephemeral output; it may be removed under `generated_artifact_policy.md`.

## Preserved Backend Boundary

All 27 boundary flags in `backend_freeze_registry.json` remain explicit false.
This closeout does not authorize backend go-live, submit, mutation, adapter
call/send, live exchange request, retry, remediation, recovery, or trading
controls. The frozen `docs/rust-cutover/release/v0_32_0_*` package is unchanged.

## Next State

- v0.32.0 remains the Backend Production Closeout baseline;
- this docs/examples governance milestone is closed, not converted into a
  backend patch release;
- retained Rust cutover history remains source-controlled audit evidence;
- future product, frontend, UX, deployment, or new-module work enters through
  the separately scoped v0.33+ intake policy.
