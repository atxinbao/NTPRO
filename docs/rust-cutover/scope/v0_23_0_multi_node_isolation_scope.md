# v0.23.0 Multi-Node Isolation Intake Scope

Date: 2026-07-03
Executor: Codex
Task: `V230-000`
GitHub issue: `#711`
Milestone: `v0.23.0`
Status: INTAKE_GATE_SATISFIED

## Summary

`v0.23.0` is the next capability track after the published
`ntpro-rust-only-v0.22.1` hardening patch. The v0.23.0 intake hard block is now
satisfied because all V221 issues are closed, the `ntpro-rust-only-v0.22.1`
GitHub Release is public, the hosted release gate succeeded for the tag commit,
and v0.22.1 strict provenance is recorded.

Plain Chinese summary: v0.23.0 的入口硬阻塞已经解除，但这不代表可以直接做大范围
runtime 改造。后续必须按 #712-#718 的单 issue、单 branch、单 PR 顺序推进，并且每
个任务继续保持 read-only / fail-closed / no-submit 边界，除非该任务明确扩大范围。

## Start Gate Evidence

```text
v0.22.1 issue set = #705, #706, #707, #708, #709, #710
v0.22.1 all issues closed = true
v0.22.1 milestone state = closed
v0.22.1 release tag = ntpro-rust-only-v0.22.1
v0.22.1 release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.22.1
v0.22.1 release published at = 2026-07-03T09:40:04Z
v0.22.1 hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28647486521
v0.22.1 hosted release gate conclusion = success
v0.22.1 hosted release gate completed at = 2026-07-03T09:38:26Z
v0.22.1 strict provenance = scripts/ai/verify_release.sh v22.1-strict-provenance
v0.22.1 tag commit = d150f7a685835eba508a5e2d9b4f832ead4d26f9
```

## V230 Issue Order

```text
#711 V230-000 v0.23.0 intake hard-blocked by v0.22.1
#712 V230-001 v0.23.0 multi-node isolation contract and scope gate
#713 V230-002 multi-account runtime identity and read-model partitioning
#714 V230-003 multi-strategy supervisor identity and isolation
#715 V230-004 multi-venue node registry and lifecycle boundary
#716 V230-005 multi-node orchestration and control-plane gating
#717 V230-006 multi-account strategy venue dashboard and observability surface
#718 V230-007 v0.23.0 release gates and strict provenance
```

`#711` only opens the intake gate. It does not implement any runtime capability.

## Capability Boundary

Allowed planning direction for v0.23.0:

- account identity and read-model partitioning;
- strategy identity and supervisor partitioning;
- venue node registry and lifecycle boundaries;
- multi-node orchestration gates;
- dashboard observability for isolated accounts, strategies, and venues;
- release gates and strict provenance for the new isolation layer.

Still forbidden unless a later V230 issue explicitly proves and gates it:

- production submit capability;
- production order mutation;
- ungated submit, cancel, retry, replace, amend, or flatten;
- automatic cancel or automatic remediation;
- strategy-driven production execution;
- implicit cross-account operation;
- implicit cross-venue operation;
- shared approval consumption across isolated nodes;
- product-grade live trading terminal claim;
- Dashboard operation controls.

## Done Boundary

V230 work must continue using one issue, one branch, and one PR. Each later
issue must include targeted tests or explicit evidence for its own boundary.
`V230-000` is complete when the intake evidence is recorded, the current release
surface points at v0.22.1, and #711 is closed after PR merge.
