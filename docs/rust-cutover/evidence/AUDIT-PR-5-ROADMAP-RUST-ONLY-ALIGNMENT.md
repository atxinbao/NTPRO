# AUDIT-PR-5 - ROADMAP Rust-Only Alignment Evidence

Date: 2026-06-06
Executor: Codex
Branch: `codex/roadmap-rust-only-alignment`

## Task

审计修复项 PR-5：清理 `ROADMAP.md` 中与 NTPRO Rust-only 公开定位冲突的
Python/PyO3/Cython 路线表述。

## Goal

- ROADMAP 与 README、Rust-only cutover contract 保持一致。
- 不再承诺 Python API、PyO3 互操作、Cython 迁移作为当前产品路线。
- v0.2 规划聚焦 Rust CLI、examples/docs、adapter support matrix、release
  delivery、runtime hardening 和 regression trace。
- 不把尚未完成的 CLI/runtime/dashboard/control API 写成已完成能力。

## Files Changed

- `ROADMAP.md`
- `docs/rust-cutover/evidence/AUDIT-PR-5-ROADMAP-RUST-ONLY-ALIGNMENT.md`

## Change Summary

已清理：

- 删除旧 “Port core to Rust” 中的 Cython replacement 和 PyO3
  interoperability commitment。
- 删除旧 “Python import resolution” ergonomics 目标。
- 删除旧 “Python API Commitment”。
- 将 roadmap 产品身份从上游 NautilusTrader 规划改成 NTPRO Rust-only
  release workspace。

合理保留：

- `Python`、`PyO3`、`Cython` 只作为 unsupported / legacy / not product
  surface 语境出现。
- `scripts/` 下本地 Python helper 脚本仍作为仓库控制和 release evidence
  工具被允许，不是产品 API。

新增：

- v0.2 priority list。
- v0.2 readiness gates。
- v0.2 out-of-scope list。
- contribution direction，明确不恢复 Python/PyO3/Cython product paths。

## Commands Run

```bash
rg -n "Python API Commitment|Ensure interoperability between Rust and Python layers using PyO3|replacing existing Cython modules|Python import resolution" ROADMAP.md
```

Result: passed; no matches.

```bash
rg -n "Python|PyO3|Cython|wheel|PyPI|dashboard|control API|live trading adapter parity" ROADMAP.md
```

Result: completed for classification. Matches are unsupported, non-goal,
out-of-scope, or future-phase wording.

```bash
scripts/ai/verify_fast.sh
```

Result: passed. Toolchain smoke and `cargo fmt --check` passed.

```bash
git diff --check
```

Result: passed.

## Behavior Impact

No runtime behavior changed. This is public documentation alignment only.

## Public API Impact

None. The PR removes misleading public roadmap promises for unsupported
Python/PyO3/Cython product paths, but it does not change code APIs.

## Migration Note

No separate migration note is required. The roadmap itself now states that
Python package/API, PyO3, Cython, wheels, and PyPI are not NTPRO product
surfaces.

## Rollback Plan

Revert this PR to restore the previous roadmap. If reverted, the public roadmap
would again conflict with the README and Rust-only cutover contract, so v0.2
release readiness should remain blocked until another roadmap cleanup lands.
