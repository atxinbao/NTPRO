# NAUDIT-003 Unignore Cache Regression Tests Evidence

Date: 2026-06-05
Executor: Codex
Task ID: NAUDIT-003
Risk: medium

## 中文摘要

这次恢复了两个之前标成 “Production bug” 的 cache 测试。它们现在不再需要
`--ignored`，会作为普通 Rust 单元测试运行。

这次没有改 cache 行为，也没有改交易语义；只是把已经能通过的回归测试重新
放回默认测试覆盖里，并更新 ignored-test 风险台账。

## Scope

Changed:

- `crates/common/src/cache/tests.rs`
- `docs/rust-cutover/verification/ignored_tests_risk_register.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/NAUDIT-003.json`

Created:

- `docs/rust-cutover/evidence/NAUDIT-003.md`

Not changed:

- No cache implementation code changed.
- No trading semantics changed.
- No other ignored tests were changed.

## Restored Tests

| Test | Previous status | New status |
| --- | --- | --- |
| `test_order_when_rejected` | `#[ignore = "Production bug: rejected orders incorrectly showing in emulated list"]` | Runs as a normal test. |
| `test_order_when_filled` | `#[ignore = "Production bug: cache state management during order lifecycle"]` | Runs as a normal test. |

## Risk Register Update

`docs/rust-cutover/verification/ignored_tests_risk_register.md` now records
these two tests under restored coverage instead of active ignored high-impact
production-bug entries. The active ignored Rust test count is updated from 30
to 28.

## Commands Run

```bash
cargo fmt -p nautilus-common
source scripts/ai/toolchain_env.sh && cargo test -p nautilus-common --lib test_order_when_rejected -- --nocapture
source scripts/ai/toolchain_env.sh && cargo test -p nautilus-common --lib test_order_when_filled -- --nocapture
rg -n '#\[ignore = "Production bug|test_order_when_rejected|test_order_when_filled|Result after NAUDIT-003|28 active ignored' crates/common/src/cache/tests.rs docs/rust-cutover/verification/ignored_tests_risk_register.md
git diff --check
scripts/ai/validate_agentflow_roles.py
scripts/ai/verify_fast.sh
```

## Results

- `test_order_when_rejected`: passed as a normal test; 1 passed, 0 ignored.
- `test_order_when_filled`: passed as a normal test; 1 passed, 0 ignored.
- `rg` check confirmed the two test names remain present and no
  `#[ignore = "Production bug..."]` attribute remains for them.
- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`; as expected,
  it skipped workspace `cargo check`, clippy, release gate, and golden trace
  gate.
- NAUDIT-003 review status: PR_OPEN in #172.

## Behavior Impact

No runtime behavior changed. The default Rust test suite now includes two
additional cache order-lifecycle regression tests.

## Public API Impact

No public API changed.

## Migration Note Status

No migration note is required.

## Rollback Plan

Revert the NAUDIT-003 PR to restore the two ignore attributes and the prior
ignored-test risk register state.
