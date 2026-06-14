# NTPRO Design Readiness Gate

Date: 2026-06-05
Executor: Codex

## 中文结论

NTPRO 进入正式产品设计或产品实现前，必须先通过本文件定义的
Design Readiness Gate。该 gate 使用严格二值判断：

```text
PASS = 可以进入下一阶段
FAIL = 不可以进入下一阶段
UNKNOWN / 未验证 = FAIL
```

不允许使用“基本通过”“部分通过”“未完全通过”作为准入状态。只要任意一项
gate 不是 `PASS`，就只能继续做 readiness hardening，不能启动 dashboard、
control API、live trading product、manual order entry 或其他正式产品实现。

## Scope

This gate sits after the Rust-only v0.1.0 release and before formal product
design and product implementation. It is stricter than the earlier roadmap
drafts: roadmap tasks can be done while the product-design gate still fails.

## Gate Matrix

| Gate | Required PASS state | Owner role | Risk |
| --- | --- | --- | --- |
| G0 State consistency | Local `main`, GitHub PR/issue state, Shrimp queue, and `.agentflow` state agree. No stale `PR_OPEN`, `REVIEW_REQUIRED`, or lease state remains for completed work. | Control & Scope Agent | medium |
| G1 Toolchain consistency | Plain `cargo` and `rustc` commands inside the repo resolve to Rust `1.95.0`, matching verification scripts. | Verification & Release Gatekeeper | medium |
| G2 Full verification | `cargo fmt --check`, workspace `cargo check`, clippy, `verify_fast`, and `verify_full` all pass without timeout or skipped required steps. | Verification & Release Gatekeeper | medium |
| G3 Core crate tests | `nautilus-core`, `nautilus-model`, `nautilus-data`, `nautilus-execution`, `nautilus-risk`, `nautilus-portfolio`, `nautilus-backtest`, and `nautilus-live` pass targeted full tests. | Verification & Release Gatekeeper | medium |
| G4 Product CLI paths | `config`, `data`, `backtest`, `sandbox`, and `live` expose real minimal Rust paths. Product commands must not be only stubs or simulated artifacts unless explicitly outside product scope. | Rust Product Surface Agent | high |
| G5 Runtime panic zero | Every `panic!`, `todo!`, and `unimplemented!` under `crates/**` is classified. Product-reachable cases are converted to explicit errors, rejections, or unsupported statuses. | Rust Core Runtime Agent | high |
| G6 Live cancellation proof | Live startup, adapter connect futures, stop, shutdown, cleanup, and half-connected state behavior have mock or fixture evidence. | Adapter & Integration Agent | high |
| G7 Ignored test closure | High-impact ignored tests are fixed, enabled, or formally release-gate exempted. No high-impact item remains `OPEN`. | Verification & Release Gatekeeper | high |
| G8 Executable trace evidence | Backtest, live/sandbox lifecycle, data source, execution order lifecycle, risk rejection, and adapter payload traces are executable and passing. | Verification & Release Gatekeeper | high |
| G9 Final design-readiness audit | A final readiness report says `Design Readiness Gate: PASS` and cites evidence for G0 through G8. | Verification & Release Gatekeeper | medium |

## Shrimp Task Mapping

| Task | Gate | Purpose |
| --- | --- | --- |
| `DRG-001` | G0 | State convergence and readiness report refresh. |
| `DRG-002` | G1 | Toolchain default convergence. |
| `DRG-003` | G2 | Full verification gate. |
| `DRG-004` | G3 | Core crates full targeted tests. |
| `DRG-005` | G4 | Real minimal CLI product paths. |
| `DRG-006` | G5 | Product-reachable panic classification and zero gate. |
| `DRG-007` | G6 | Live adapter cancellation proof closure. |
| `DRG-008` | G7 | High-impact ignored tests closure. |
| `DRG-009` | G8 | Executable trace gate. |
| `DRG-010` | G9 | Final design-readiness audit. |

## Execution Order

```text
NAUDIT-006
  -> DRG-001
  -> DRG-002
  -> DRG-003
  -> DRG-004
  -> DRG-006
  -> DRG-007
  -> DRG-008
  -> DRG-005
  -> DRG-009
  -> DRG-010
```

`DRG-005` is intentionally placed after runtime risk closure because real CLI
product paths should not hide unresolved runtime panic, cancellation, or ignored
test risk.

## Non-Goals

- Do not start dashboard UI implementation from this gate.
- Do not add control API endpoints from this gate.
- Do not connect to real exchanges for readiness evidence.
- Do not treat simulated demos as product runtime paths.
- Do not create release tags or GitHub Releases from readiness tasks.

## Promotion Rule

Formal product design can start only when `DRG-010` records:

```text
Design Readiness Gate: PASS
```

Before that point, allowed work is limited to readiness hardening, evidence,
documentation, and explicitly scoped product-path implementation tasks.
