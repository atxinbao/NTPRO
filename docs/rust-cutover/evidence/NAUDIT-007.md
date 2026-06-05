# NAUDIT-007 Unsafe And Plugin Audit Register Evidence

Date: 2026-06-05
Executor: Codex
Task ID: NAUDIT-007
Risk: medium

## 中文摘要

这次没有改 plugin loader，也没有改 actor registry 或 live node 行为。

这次只是把风险台账建起来：哪些 unsafe / FFI / ABI / plug-in loading
点已经有文档限制，哪些点在插件被当作稳定产品能力前必须补测试或补
release gate。结论是：v0.2 里 plugin 只能继续按 early alpha / unstable
看待，不能写成稳定可生产使用的扩展能力。

## Scope

Changed:

- `docs/rust-cutover/verification/unsafe_plugin_audit_register.md`
- `docs/rust-cutover/evidence/NAUDIT-007.md`
- `docs/rust-cutover/release/v0_2_readiness_report.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/NAUDIT-007.json`

Not changed:

- No runtime code changed.
- No plug-in loader behavior changed.
- No actor registry behavior changed.
- No live node startup/shutdown behavior changed.
- No ABI version, manifest structure, or feature flag changed.

## Inventory Summary

The register covers:

- actor registry `UnsafeCell` / mutable aliasing risk;
- `libloading` path trust and cdylib load side effects;
- optional SHA-256 plug-in verification;
- ABI and manifest compatibility;
- intentional no-`dlclose` / no-hot-reload behavior;
- panic boundary and infallible thunk abort behavior;
- live node startup-only plug-in lifecycle;
- manual/platform-specific cdylib smoke coverage;
- FFI primitive ownership and host context lifetime.

## Commands Run

```bash
rg -n "unsafe|UnsafeCell|extern \"C\"|libloading|ManuallyDrop|catch_unwind|sha256|dlclose|ABI|ValidatedPluginManifest" \
  crates/common/src/actor/registry.rs crates/plugin/src crates/live/src/node.rs -g '*.rs'
rg -n "plug-in|plugin|ABI|unsafe|SHA-256|sha256|dlclose|hot reload|early alpha" \
  crates/plugin/README.md docs/rust-cutover/verification/ignored_tests_risk_register.md
git diff --check
scripts/ai/validate_agentflow_roles.py
scripts/ai/verify_fast.sh
```

## Results

- Unsafe / plug-in boundary source inventory completed.
- Source inventory count: 747 matching lines across actor registry, plug-in
  sources, and live node plug-in wiring.
- Documentation inventory count: 35 matching lines across the plug-in README
  and ignored-test risk register.
- Risk register created.
- Follow-up task candidates recorded.
- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`; as expected,
  it skipped workspace `cargo check`, clippy, release gate, and golden trace
  gate.

## Behavior Impact

No behavior changed. This PR is audit documentation and task-state evidence
only.

## Public API Impact

No public API changed.

## Migration Note Status

No migration note is required. The public product decision is documented in
`docs/rust-cutover/verification/unsafe_plugin_audit_register.md`: plug-ins are
not stable product functionality for v0.2.

## Review And Merge Status

PR opened: <https://github.com/atxinbao/NTPRO/pull/174>

Status: `PR_OPEN`.

## Rollback Plan

Revert the NAUDIT-007 PR to remove the audit register, evidence, and related
agentflow state update. Since no runtime code changed, rollback has no runtime
or data migration step.
