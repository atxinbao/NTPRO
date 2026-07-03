# NTPRO Rust-only v0.22.1 Release Notes

Date: 2026-07-03
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.22.1`
Release name: `NTPRO Rust-only v0.22.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.22.1`
Base release: `ntpro-rust-only-v0.22.0`

## Scope

v0.22.1 is the Trader Terminal Workbench hardening patch. It keeps the
v0.22.0 Workbench/runtime bridge scope, but closes the release-governance and
runtime-boundary gaps found after v0.22.0.

Plain Chinese summary: v0.22.1 是 Trader Terminal Workbench 的补丁发布。它不
新增交易能力，不启动 v0.23.0，也不把 Workbench 宣称为完整 executable read-model
runtime。它修复的是禁用边界、read-model replay 证据、发布顺序治理、Workbench render
smoke 和 release provenance。

This release is read-only first. This release is not a product-grade live trading terminal. This release does not add submit capability.

## Included Evidence

```text
V221-001 release closeout milestones and evidence ledger
V221-002 required-false runtime operation boundary hardening
V221-003 executable read-model replay expansion
V221-004 gate-before-publish release governance
V221-005 Workbench artifact render smoke and read-only regression
V221-006 v0.22.1 release gates and strict provenance
```

## Hardening Summary

v0.22.1 adds these patch-level protections:

```text
required_false_runtime_boundary = enforced
missing_operation_boundary_field = fail_closed
true_operation_boundary_field = fail_closed
read_model executable_replay rows = 28
read_model schema_only_scoped rows = 4
workbench render smoke = required
gate_before_publish = required
strict provenance = required
v0.23.0 hard block = still active until this release is public
```

## Release Gates And Strict Provenance

The v0.22.1 release package is verified by:

```text
scripts/ai/verify_release.sh v22-runtime-boundary-tests
scripts/ai/verify_release.sh v21.1-read-model-projection-replay
scripts/ai/verify_v22_workbench_render_smoke.sh
scripts/ai/verify_release.sh release-publish-after-gate
scripts/ai/verify_release.sh v22.1-release-gates
scripts/ai/verify_release.sh v22.1-strict-provenance
scripts/ai/verify_v22_1_release_gates.sh
scripts/ai/verify_v22_1_strict_provenance.sh
```

The strict provenance gate writes:

```text
target/ntpro-v221/v0_22_1_strict_release_manifest.json
```

Public GitHub Release publication must happen after the hosted
`Rust Cutover Release Gate` succeeds for the same `ntpro-rust-only-v0.22.1`
tag commit. Publication uses:

```text
scripts/ai/publish_ntpro_release_after_gate.sh
```

## Boundary

v0.22.1 explicitly does not include:

- product-grade live trading terminal readiness;
- complete executable read-model runtime coverage;
- new production submit capability;
- production order mutation;
- ungated submit, cancel, retry, replace, amend, or flatten;
- manual operation entry that can mutate live state;
- automatic cancel, repair, alert, audit, provenance, risk, or operation action;
- funds transfer or account configuration mutation;
- execution algorithm routing;
- strategy-driven production execution;
- multi-account production execution expansion;
- multi-strategy production execution expansion;
- multi-venue production execution expansion;
- listenKey lifecycle;
- real-funds proof in CI;
- Dashboard order, approval, cancel, retry, submit, replace, amend, flatten,
  fill, risk, or order-ticket controls.

## Validation

Use:

```bash
scripts/ai/verify_release.sh v22.1-release-gates
scripts/ai/verify_release.sh v22.1-strict-provenance
```

This release validates the Workbench hardening evidence and release
provenance only. The next capability track is `v0.23.0`, and it remains
blocked until `v0.22.1` is published and issue `#710` is closed with release
evidence.
