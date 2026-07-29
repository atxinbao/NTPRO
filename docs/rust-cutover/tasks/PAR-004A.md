# PAR-004A TWAP High-Precision Remainder Fixture

Date: 2026-07-29
Executor: Codex
GitHub issue: #1185
Risk: medium
Classification: separately-scoped-test-maintenance
Status: LOCAL VALIDATION PASSED / REVIEW_REQUIRED

## Goal

Make the TWAP remainder schedule regression test executable under both standard
and high precision without changing production scheduling behavior.

中文摘要：`nautilus-trading` 的本地 `high-precision` feature 会转发到
`nautilus-model/high-precision`，但审计复现命令直接启用了后者。Cargo
feature 不会从依赖反向激活调用 crate 的本地 feature，因此测试的本地
`cfg(feature = "high-precision")` 仍为 false，并错误使用标准精度 raw
常量。本任务将断言改为固定点分辨率、近似三等分和总量守恒契约。

## Scope

- update only the TWAP remainder test and its imports;
- assert equal regular slices;
- assert each regular slice approximates one third within the active fixed
  precision;
- assert the one-unit remainder matches the active fixed-point resolution;
- assert three regular slices plus the remainder exactly conserve total raw
  quantity;
- add PAR-004A evidence.

## Non-Goals

- no production TWAP scheduling change;
- no public API or migration change;
- no edit to frozen `docs/rust-cutover/release/v0_32_0_*` files;
- no submit, mutation, adapter send, live exchange, retry, remediation, or
  trading-control capability authorization.

## Acceptance

- the targeted test passes in standard precision;
- the targeted test and complete `nautilus-trading` suite pass with
  `nautilus-model/high-precision`;
- trading Clippy, current governance, backend freeze, and fast smoke pass;
- independent review and hosted checks approve the change.
