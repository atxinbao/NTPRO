# NTPRO v0.2 Readiness Report

Date: 2026-06-05
Executor: Codex
Task ID: NQA-001, DRG-001, DRG-002, DRG-003

## DRG-001 Update

Date: 2026-06-05
Executor: Codex

```text
Design Readiness Gate: FAIL
G0 State consistency: PASS
G1 Toolchain consistency: PASS
G2 Full verification: PASS
G3-G9: FAIL / not yet executed
```

DRG-001 只完成 state convergence。当前 GitHub open PR/issue 为空，Shrimp
只有 `DRG-001` in progress，`.agentflow` 已经没有历史 stale
`PR_OPEN` / `REVIEW_REQUIRED` / `QA_REQUIRED` 状态。已合并任务和旧 lease
状态已经收口。

DRG-002 随后修正了本地 shell 的普通 `cargo` / `rustc` 解析路径：
NTPRO 目录内解析到 Rust `1.95.0`，其他目录仍回落到原 Homebrew 工具链。

DRG-003 完成了完整本地验证门禁。`cargo fmt --check`、带
`arrow,ffi,high-precision,streaming,defi` feature 的 workspace
`cargo check` 和 `cargo clippy -D warnings`、`scripts/ai/verify_fast.sh`、
`scripts/ai/verify_full.sh` 均已通过。`verify_full` 覆盖 fast checks、
clippy、workspace Rust tests、golden trace validation、golden trace replay
tests 和 `cargo doc --workspace --no-deps`。为使 full gate 在当前 clippy
策略下通过，本轮只做了小范围机械修正：CLI 内部 helper 改为借用参数、
matching-engine 测试移除冗余 clone、live startup helper 参数按控制项和
事件接收器分组。未修改交易语义、CLI 产品承诺、adapter 行为或 public API。

这不代表 v0.2 可以启动正式产品设计。按照
`docs/rust-cutover/design_readiness_gate.md`，只要 G3 到 G9 任意一项没有
明确 `PASS`，最终 Design Readiness Gate 仍是 `FAIL`。

下一步执行顺序：

```text
DRG-002 -> DRG-003 -> DRG-004 -> DRG-006 -> DRG-007 -> DRG-008 -> DRG-005 -> DRG-009 -> DRG-010
```

## 中文结论

这次 QA 收口只确认 v0.2 roadmap 里 9 个已经交付的规划、文档和合同类
任务可以从 `QA_REQUIRED` 进入 `DONE`。这不等于 v0.2 已经可以发布。

当前判断：v0.2 暂不建议打 tag。原因是新审计 backlog 里还有公开面和运行面
问题需要先处理，尤其是 Python package metadata、CLI 能力口径、被 ignore 的
生产缺陷测试，以及若要声明更强 runtime 能力时必须处理的 panic/cancellation
风险。

## Closeout Scope

NQA-001 reviewed these tasks:

- `NBIN-001`
- `NTRACE-001`
- `NARCH-001`
- `NARCH-006`
- `NARCH-002`
- `NARCH-003`
- `NARCH-004`
- `NARCH-005`
- `NDASH-001`

This report does not implement CLI runtime wiring, dashboard UI, control API
endpoints, adapter behavior, trading semantics, or release tagging.

## QA Decisions

| Task | Decision | Merged PR | Evidence | QA rationale |
| --- | --- | ---: | --- | --- |
| `NBIN-001` | `DONE` | #138 | `docs/rust-cutover/evidence/NBIN-001.md` | Install/run path decisions are documented, `cargo install --path` evidence is recorded, and no release artifact was published. |
| `NTRACE-001` | `DONE` | #139 | `docs/rust-cutover/evidence/NTRACE-001.md` | Trace/performance expansion plan exists and separates required, deferred, and future evidence without changing runtime behavior. |
| `NARCH-001` | `DONE` | #140 | `docs/rust-cutover/evidence/NARCH-001.md` | Rust-only architecture map exists and records follow-up unknowns without refactoring crates. |
| `NARCH-006` | `DONE` | #141 | `docs/rust-cutover/evidence/NARCH-006.md` | Module boundary audit exists and separates refactor candidates from executable changes. |
| `NARCH-002` | `DONE` | #142 | `docs/rust-cutover/evidence/NARCH-002.md` | Module contracts exist and distinguish current behavior from future dashboard needs. |
| `NARCH-003` | `DONE` | #143 | `docs/rust-cutover/evidence/NARCH-003.md` | Node lifecycle contract exists and explicitly marks pause/resume states as future contract states. |
| `NARCH-004` | `DONE` | #144 | `docs/rust-cutover/evidence/NARCH-004.md` | Future observability model exists and scopes out secrets, raw payloads, and mutable engine access. |
| `NARCH-005` | `DONE` | #145 | `docs/rust-cutover/evidence/NARCH-005.md` | Control API contract exists and records that no runtime control endpoint was added. |
| `NDASH-001` | `DONE` | #146 | `docs/rust-cutover/evidence/NDASH-001.md` | Dashboard MVP scope contract exists and records that dashboard implementation is still out of scope. |

## Tag Readiness

Status: not tag-ready.

The 9 closeout tasks are complete as scoped, but v0.2 should not be tagged until
the audit backlog is reduced or explicitly scoped by the release gatekeeper.

## Blockers And Follow-Up Mapping

| Follow-up | Risk | Readiness impact | Required decision before v0.2 tag |
| --- | --- | --- | --- |
| `NAUDIT-001` | critical | Root Python package metadata conflicts with Rust-only public positioning. | Complete before tag, or record an explicit release-gate exception. |
| `NAUDIT-002` | medium | CLI commands and sandbox artifacts must not overstate stubbed or simulated capability. | Complete before tag. |
| `NAUDIT-003` | medium | Passing production-bug cache tests should be restored to normal regression coverage. | Complete before tag unless a failed rerun creates a new blocker. |
| `NAUDIT-005` | medium | PostgreSQL cache adapter status must be visible as supported, experimental, or unsupported. | Complete or explicitly defer from v0.2 product claims. |
| `NAUDIT-007` | medium | Plugin/unsafe areas need a register before plugin functionality is productized. | Register created; plug-ins remain early alpha / unstable unless later gates pass. |
| `NAUDIT-004` | high | Product-reachable runtime panic paths are high-risk runtime hardening work. | Human/gatekeeper decision: v0.2 blocker or scoped follow-up. |
| `NAUDIT-006` | high | Live adapter cancellation proof is high-risk adapter/runtime evidence work. | Human/gatekeeper decision: v0.2 blocker or scoped follow-up. |

## Release Recommendation

Do not create a v0.2 tag yet.

Recommended next execution order:

1. `NAUDIT-002` - CLI capability matrix and stub honesty cleanup.
2. `NAUDIT-003` - unignore passing production-bug cache tests.
3. `NAUDIT-005` - PostgreSQL cache adapter support classification.
4. `NAUDIT-007` - unsafe and plugin audit register.
5. `NAUDIT-001` - Python package metadata cleanup, after explicit critical-risk
   approval because it touches root package metadata and gate behavior.
6. Decide whether `NAUDIT-004` and `NAUDIT-006` are v0.2 blockers or later
   high-risk hardening tasks.

## Behavior And API Impact

No runtime behavior changed. No Rust public API changed. No Python/PyO3/Cython
surface was added. No tag or GitHub Release was created.

## Rollback Plan

Revert the NQA-001 PR to restore the 9 reviewed tasks to `QA_REQUIRED` and
remove this readiness report and NQA-001 evidence.
