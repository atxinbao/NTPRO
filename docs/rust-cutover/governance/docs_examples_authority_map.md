# Docs And Examples Authority Map

Date: 2026-07-15
Executor: Codex
Baseline: `ntpro-rust-only-v0.32.0`
Classification: `baseline-preserving-governance`
Status: ACTIVE

## Decision

Documentation and examples cleanup occurs after the v0.32.0 backend freeze.
The cleanup may remove unsupported current-route material, but it must preserve
the release and cutover audit chain. No cleanup item authorizes runtime,
trading, adapter, retry, remediation, or product capability.

Plain Chinese summary: 本文件确定 `docs/` 和 `examples/` 的清理边界。完整的 Rust
cutover 审计链继续保留，v0.32.0 发布文件保持冻结；当前公开文档中的 Python 遗留内容
按独立 issue 清理，Rust examples 保留并修正失效引用。本地缓存和可重建生成物可以删除，
但不能把历史证据当作缓存批量清除。

## Authority Classes

### Frozen Baseline

- `docs/rust-cutover/release/v0_32_0_*`
- the published `ntpro-rust-only-v0.32.0` tag and GitHub Release identity;
- the explicit-false capability boundary registered by
  `backend_freeze_registry.json`.

These surfaces are immutable during routine documentation cleanup. Corrections
belong in governance errata, not in the frozen files.

### Retained Audit Chain

- `docs/rust-cutover/tasks/`;
- `docs/rust-cutover/evidence/`;
- `docs/rust-cutover/release/` outside an explicitly approved future task;
- `docs/rust-cutover/governance/`, `golden_trace/`, `migration/`, `product/`,
  `scope/`, and the remaining cutover contracts and indexes.

These paths are source-controlled evidence or active contract inputs. They are
not generated caches and must not be deleted by directory cleanup.

### Retained Canonical Examples

- `examples/rust/` and its tracked configuration, backtest, live, and README
  surfaces.

DEXG-002 may repair stale paths and status metadata. It must not add execution
authority or silently delete examples referenced by product contracts.

### Rewrite Or Retire

- Python legacy tutorials and affected tutorial indexes;
- Python-specific how-to and developer-guide pages;
- integration pages containing unqualified Python runtime instructions;
- concept and public pages linking to `/docs/python-api-latest/`.

DEXG-004 through DEXG-006 own these decisions. A page is rewritten when a
current Rust/config/CLI authority exists; otherwise it is retired with an
explicit migration record instead of invented replacement behavior.

### Removable After Dependency Review

- `docs/api_reference/`, the legacy upstream Python API appendix;
- the obsolete `docs-python` build entry and dependencies used only by it;
- tutorial media that becomes unreferenced after its owning page is removed.

DEXG-003 owns the API appendix and build entry. DEXG-004 owns conditional
tutorial asset deletion. No asset is removed before a repository-wide reference
check in its owning issue.

### Ephemeral Local Output

- `.DS_Store` under `docs/` or `examples/`;
- `__pycache__/` and `*.pyc`;
- `/target/`, `/release-publication-evidence/`, and `/graphify-out/` as further
  classified by `generated_artifact_policy.md`.

These paths are not release authority. At DEXG-001 closeout, `docs/` and
`examples/` contain no `.DS_Store`; later Finder or validation runs may recreate
local artifacts, which may be removed without a tracked change.

## Execution Map

| Issue | Owned surface | Dependency |
| --- | --- | --- |
| `#1080` / DEXG-001 | authority map and cleanup boundaries | none |
| `#1081` / DEXG-002 | `examples/rust/` path and status integrity | DEXG-001 |
| `#1082` / DEXG-003 | `docs/api_reference/` and `docs-python` | DEXG-001 |
| `#1083` / DEXG-004 | tutorials, how-to, developer guide, conditional media | DEXG-003 |
| `#1084` / DEXG-005 | integration documentation | DEXG-003 |
| `#1085` / DEXG-006 | concepts, public docs, stale Python API links | DEXG-003; coordinated with DEXG-005 |
| `#1086` / DEXG-007 | supported docs build and link gates | DEXG-002, DEXG-004, DEXG-005, DEXG-006 |
| `#1087` / DEXG-008 | live and source-controlled closeout | DEXG-007 and all preceding tasks |

## Guardrails

1. Every tracked cleanup uses one issue, one branch, and one PR.
2. Directory-wide deletion is allowed only for a surface explicitly classified
   as removable by its owning issue.
3. `docs/rust-cutover/` is retained as an audit system; only task-scoped edits
   may add governance, task, evidence, migration, or closeout records.
4. Supported replacement text must cite current repository authority. Missing
   Rust behavior is documented as unavailable, not implemented by documentation.
5. DEXG-007 must validate the final supported surface without exclusions that
   hide retired Python API links.
