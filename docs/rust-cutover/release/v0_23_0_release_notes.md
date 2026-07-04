# NTPRO Rust-only v0.23.0 Release Notes

Date: 2026-07-03
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.23.0`
Release name: `NTPRO Rust-only v0.23.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.23.0`
Previous release: `ntpro-rust-only-v0.22.1`

## Scope

v0.23.0 is the Multi-Account / Multi-Strategy / Multi-Venue Node Isolation
release. It adds scoped identity, replay, gate, and read-only observability
evidence for account, strategy, venue node, orchestration/control-plane, and
Dashboard boundaries.

Plain Chinese summary: v0.23.0 的发布重点是隔离边界和只读观测，不是开放更多交易
按钮。它证明多账户、多策略、多 venue node 的数据、证据、日志、Dashboard 视图和未来
owner-approved control-plane 不能串线；缺失或冲突身份会 fail closed 或 degraded
unavailable。这个版本仍不新增 submit 能力，不开放 production order mutation，不把
Workbench/Dashboard 宣称为产品级实盘交易终端。

This release does not add submit capability.
This release is not a product-grade live trading terminal.

## Included Evidence

```text
V230-000 v0.23.0 intake gate and v0.22.1 dependency proof
V230-001 multi-node isolation contract
V230-002 multi-account runtime identity and read-model partitioning
V230-003 multi-strategy supervisor identity and isolation
V230-004 multi-venue node registry and lifecycle boundary
V230-005 multi-node orchestration and control-plane gating
V230-006 Dashboard / Workbench observability surface
V230-007 v0.23.0 release gates and strict provenance
```

## Capability Summary

```text
multi_account_isolation = true
multi_strategy_isolation = true
multi_venue_node_isolation = true
cross_node_read_model_aggregation = true
read_only_dashboard_observability = true
owner_approved_control_contract_defined = true
gate_before_publish = required
strict provenance = required
read_model executable_replay rows = 45
read_model schema_only_scoped rows = 4
release manifest cases = 100
release executable_replay cases = 95
release schema_only_scoped cases = 5
capability class = evidence / replay / readonly observability only
production multi-node runtime implementation = not included
v0.24.0 entry = future contract and gated implementation only
```

## Release Gates And Strict Provenance

The v0.23.0 release package is verified by:

```text
scripts/ai/verify_release.sh v23-release-gates
scripts/ai/verify_release.sh v23-strict-provenance
scripts/ai/verify_v23_release_gates.sh
scripts/ai/verify_v23_strict_provenance.sh
scripts/ai/verify_release.sh v23.1-gate-phase-split
scripts/ai/verify_v23_dashboard_observability_smoke.sh
scripts/ai/verify_release.sh release-publish-after-gate
```

Post-release closeout cleanup is verified by:

```text
scripts/ai/verify_release.sh v23.1-stale-provenance-cleanup
scripts/ai/verify_release.sh v23.1-gate-phase-split
scripts/ai/verify_release.sh v23.1-evidence-replay-only-boundary
```

The strict provenance gate writes:

```text
target/ntpro-v230/v0_23_0_strict_release_manifest.json
```

Public GitHub Release publication must happen after the hosted
`Rust Cutover Release Gate` succeeds for the same `ntpro-rust-only-v0.23.0`
tag commit. Publication uses:

```text
scripts/ai/publish_ntpro_release_after_gate.sh
```

## Boundary

v0.23.0 explicitly does not include:

- product-grade live trading terminal readiness;
- complete executable read-model runtime coverage;
- new production submit capability;
- production order mutation;
- strategy-driven production execution;
- automatic cancel;
- automatic remediation;
- cross-account implicit operation;
- cross-strategy implicit operation;
- cross-venue implicit operation;
- shared approval consumption across isolated nodes;
- ungated submit, cancel, retry, replace, amend, or flatten;
- manual operation entry that can mutate live state;
- Dashboard operation controls;
- Dashboard submit/cancel/retry/replace/amend/flatten/order-ticket controls;
- real-funds proof in CI.

## Validation

Use:

```bash
scripts/ai/verify_release.sh v23-release-gates
scripts/ai/verify_release.sh v23-strict-provenance
```

This release validates multi-node isolation and read-only observability
evidence. The next capability track is `v0.24.0`, and it remains undefined
until a later scoped issue set publishes its own contract and release gates.
