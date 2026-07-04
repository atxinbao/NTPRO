# NTPRO Rust-only v0.23.1 Release Notes

Date: 2026-07-04
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.23.1`
Release name: `NTPRO Rust-only v0.23.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.23.1`
Base release: `ntpro-rust-only-v0.23.0`

## Scope

v0.23.1 is a patch closeout release for the v0.23.0
Multi-Account / Multi-Strategy / Multi-Venue Node Isolation line. It converts
the V231 post-release fixes into release-blocking gates and strict provenance
for the next published source tag.

Plain Chinese summary: v0.23.1 是 v0.23.0 发布后的治理收口补丁。它把 closeout
evidence、stale provenance 清理、pre-release/post-release gate 拆分、evidence /
replay-only 边界、publication evidence audit path 都纳入发布阻断条件。它不新增交易
能力，不启动 v0.24.0，也不把 Workbench 或 Dashboard 宣称为产品级实盘终端。

This release is a patch closeout. This release does not add submit capability.
This release is not a product-grade live trading terminal. This release does
not add production order mutation, Dashboard operation controls, or a runtime
execution algorithm.

## Included Evidence

```text
V231-001 release closeout evidence backfill
V231-002 stale provenance cleanup
V231-003 pre-release vs post-release gate phase split
V231-004 evidence/replay-only boundary hardening
V231-005 publication evidence audit path
V231-006 v0.23.1 release gates and strict provenance
```

## Patch Hardening Summary

```text
patch_closeout_only = true
v0.23.0 closeout facts = required
candidate / pending / in-progress marker cleanup = required
pre_release_gate_semantics = separated
post_release_publication_evidence = separated
evidence_replay_only_boundary = required
publication evidence strategy = source_tree_plus_github_remote
v0.24.0 start gate = blocked until v0.23.1 release evidence is published
v0.24.0 remains blocked
new_submit_capability = false
production_order_mutation_allowed = false
dashboard_operation_controls_enabled = false
product_grade_live_trading_terminal_claim = false
```

## Release Gates And Strict Provenance

The v0.23.1 release package is verified by:

```text
scripts/ai/verify_release.sh v23.1-release-closeout-evidence
scripts/ai/verify_release.sh v23.1-stale-provenance-cleanup
scripts/ai/verify_release.sh v23.1-gate-phase-split
scripts/ai/verify_release.sh v23.1-evidence-replay-only-boundary
scripts/ai/verify_release.sh v23.1-publication-evidence-audit-path
scripts/ai/verify_release.sh v23.1-release-gates
scripts/ai/verify_release.sh v23.1-strict-provenance
scripts/ai/verify_v23_1_release_gates.sh
scripts/ai/verify_v23_1_strict_provenance.sh
scripts/ai/verify_release.sh release-publish-after-gate
scripts/ai/publish_ntpro_release_after_gate.sh
```

The strict provenance gate writes:

```text
target/ntpro-v231/v0_23_1_strict_release_manifest.json
```

Public GitHub Release publication must happen after the hosted
`Rust Cutover Release Gate` succeeds for the same `ntpro-rust-only-v0.23.1`
tag commit. Publication uses:

```text
scripts/ai/publish_ntpro_release_after_gate.sh
```

## Boundary

v0.23.1 explicitly does not include:

- v0.24.0 execution algorithms implementation;
- product-grade live trading terminal readiness;
- complete executable read-model runtime coverage;
- new production submit capability;
- production order mutation;
- ungated submit, cancel, retry, replace, amend, or flatten;
- manual operation entry that can mutate live state;
- automatic cancel, retry, remediation, repair, alert, audit, provenance, risk,
  or operation action;
- strategy-driven production execution;
- cross-account, cross-strategy, cross-venue, or cross-node implicit operation;
- shared approval consumption;
- listenKey lifecycle;
- real-funds proof in CI;
- Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or
  order-ticket controls.

## v0.24.0 Start Gate

`v0.24.0` remains blocked until all V231 issues are closed and v0.23.1 release
evidence is published. The v0.24.0 line starts from future contract and gated
implementation work only; it does not inherit production submit, production
order mutation, automatic execution, shared approval consumption, or Dashboard
operation controls from v0.23.1.

## Validation

Use:

```bash
scripts/ai/verify_release.sh v23.1-release-gates
scripts/ai/verify_release.sh v23.1-strict-provenance
```

This release validates patch closeout governance, strict provenance, and the
v0.24.0 start gate only. It does not expand the v0.23.0 runtime capability
surface.
