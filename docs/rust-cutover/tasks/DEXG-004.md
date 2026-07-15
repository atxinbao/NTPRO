# DEXG-004 Retire Legacy Python Guides

Date: 2026-07-15
Executor: Codex
GitHub issue: #1083
Milestone: post-backend-docs-examples-governance
Status: READY FOR PR

## Goal

Remove Python-first tutorials, live configuration how-to content, and the
legacy Python developer guide from the active Rust-only documentation route.

Plain Chinese summary: 本任务删除 5 个 Python-first 页面和只属于其中 3 个教程的
12 张图片，修复所有入站导航，并保留迁移说明。没有等价 Rust contract 的内容不做
虚假重写。

## Dependency

DEXG-003 / #1082 is merged and closed.

## Scope

Included:

- remove the three identified Python strategy tutorials;
- remove the Python live configuration how-to and developer guide;
- remove only assets that become unreferenced;
- repair indexes and inbound links;
- add a migration tombstone and explicit backend-freeze routing.

Not included:

- rewriting integration or concept content beyond links affected by deletion;
- implementing replacement runtime behavior;
- changing frozen v0.32.0 release files or capability.

## Acceptance Criteria

- the five retired pages have no active link;
- the 12 deleted images have no remaining reference;
- every retained tutorial image remains referenced;
- tutorial, how-to, and developer indexes expose no Python product route;
- backend freeze, fast smoke, and link-target checks pass.

## Validation

```bash
rg -n 'fx_mean_reversion_ax|gold_book_imbalance_ax|grid_market_maker_bitmex|configure_live_trading|developer_guide/python' docs \
  --glob '!docs/rust-cutover/tasks/**' \
  --glob '!docs/rust-cutover/evidence/**' \
  --glob '!docs/rust-cutover/migration/**' \
  --glob '!docs/rust-cutover/governance/**' \
  --glob '!docs/rust-cutover/inventory/**'
scripts/ai/check_backend_freeze_baseline.sh
scripts/ai/verify_fast.sh
git diff --check
```
