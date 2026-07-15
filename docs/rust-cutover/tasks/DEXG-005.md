# DEXG-005 Integration Documentation Rust Authority

Date: 2026-07-15
Executor: Codex
GitHub issue: #1084
Milestone: post-backend-docs-examples-governance
Status: READY FOR PR

## Goal

Normalize every integration page containing Python snippets so venue facts are
retained but current NTPRO authority is unambiguously Rust-only.

Plain Chinese summary: 15 个 integration 页同时包含有效的 venue 协议信息和大量
上游 Python snippet。直接删除代码块会破坏订单矩阵、symbology、rate-limit 等上下文，
所以本任务统一把这些 snippet 标记为 retired lineage，并将实现依据固定到对应 Rust
adapter crate、tests、fixtures 和 bounded product contract。

## Dependency

DEXG-003 / #1082 is merged and closed.

## Scope

Included:

- add a uniform Rust-only authority boundary to all 15 affected pages;
- define integration index status as implementation status, not production
  authorization;
- remove active optional-Python-binding claims from Deribit and Tardis;
- route Tardis replay and CSV guidance to its Rust binary/adapter surface;
- verify every affected page maps to a tracked Rust adapter crate.

Not included:

- changing adapter implementation, protocol behavior, or capability matrices;
- deleting valid venue protocol and symbology lineage;
- editing concept links owned by DEXG-006;
- authorizing production execution.

## Acceptance Criteria

- every integration page with a Python fence has the exact Rust-only authority
  warning near the title;
- no page claims NTPRO exposes optional Python product bindings;
- every affected page maps to a tracked Rust adapter crate;
- the index states that stable is not production authorization;
- backend freeze and fast smoke checks pass.

## Validation

```bash
for page in $(rg -l '```python' docs/integrations --glob '*.md'); do
  sed -n '1,16p' "$page" | rg -q ':::warning\[Rust-only authority\]'
done
test -f crates/adapters/architect_ax/Cargo.toml
test -f crates/adapters/interactive_brokers/Cargo.toml
test -f crates/adapters/tardis/Cargo.toml
! rg -n -i 'optional Python bindings|Python-based workflows' docs/integrations
scripts/ai/check_backend_freeze_baseline.sh
scripts/ai/verify_fast.sh
git diff --check
```
