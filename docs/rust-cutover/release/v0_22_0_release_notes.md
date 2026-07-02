# NTPRO Rust-only v0.22.0 Release Notes

Date: 2026-07-02
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.22.0`
Release name: `NTPRO Rust-only v0.22.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.22.0`
Base release: `ntpro-rust-only-v0.21.1`

## Scope

v0.22.0 is the Trader Terminal Workbench release. It turns the v0.21.1
canonical Unified Read Model runtime bridge into a read-only first workbench
for account, position, order, fill, risk, alerts, audit, provenance, and
gated manual operation-entry evidence.

This release is read-only first. This release is not a product-grade live trading terminal. This release does not add submit capability.

Plain Chinese summary: v0.22.0 是 Trader Terminal workbench 发布线。它把
v0.21.1 的 canonical read model runtime 展示为交易员工作台：账户、持仓、订单、
成交、风控、告警、审计、provenance 和人工操作入口合同都可见。但它仍然是
read-only first。任何真实 submit/cancel/retry/replace/amend/flatten 都没有开放；
人工操作入口只是 disabled/gated preview，必须先有 owner approval、risk gate 和
audit gate。v0.22.0 不是产品级实盘交易终端。

## Workbench Surface

The release includes the V220 evidence chain:

```text
V220-000 v0.22 scope decision and v0.21.1 dependency gate
V220-001 Trader Terminal read-only workbench shell and navigation
V220-002 Account and position workbench panels
V220-003 Order and fill workbench panels
V220-004 Risk alerts audit and provenance drill-down panels
V220-005 Gated manual operation entry contract
V220-006 Trader Terminal runtime degradation and boundary tests
V220-007 v0.22 release gates strict provenance and workbench evidence
```

The workbench reads from the existing local read-model artifact path and keeps
missing or invalid runtime evidence visible:

```text
source artifact = v0_21/unified_read_model_snapshot.json
source snapshot field = read_model_runtime
read_only_first = required
gated_operation_boundary = required
owner approval = required before any future real operation
risk gate = required before any future real operation
audit gate = required before any future real operation
```

## Release Gates And Strict Provenance

V220-007 adds the final v0.22.0 release gates:

```text
scripts/ai/verify_release.sh v22-runtime-boundary-tests
scripts/ai/verify_release.sh v22-release-gates
scripts/ai/verify_release.sh v22-strict-provenance
scripts/ai/verify_release_strict.sh v22
scripts/ai/verify_v22_release_gates.sh
scripts/ai/verify_v22_strict_provenance.sh
```

The release gate fails closed if any V220 evidence file is missing, if any
required workbench release note is missing, if the v0.21.1 base release is not
published, if V220 issue closeout is incomplete during the release gate, or if
the release body stops stating the read-only first and gated-operation
boundary.

The strict provenance gate records the tag, source commit, source tree,
release notes, readiness report, release manifest, scope decision, V220
evidence set, and workbench evidence inputs in a generated manifest:

```text
target/ntpro-v220/v0_22_strict_release_manifest.json
```

## Boundary

v0.22.0 explicitly does not include:

- product-grade live trading terminal readiness;
- new production submit capability;
- production order mutation;
- ungated submit/cancel/retry/replace/amend/flatten;
- automatic cancel;
- automatic remediation;
- retry, replace, amend, correction, or flatten routes;
- strategy-driven production execution;
- multi-account production execution expansion;
- multi-venue production execution expansion;
- listenKey creation, keepalive, or close lifecycle;
- real-funds proof in CI;
- Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or order-ticket controls;
- manual operation entry that can mutate live state without owner approval, risk gate, and audit gate.

## Validation

Use:

```bash
scripts/ai/verify_release.sh v22-runtime-boundary-tests
scripts/ai/verify_release.sh v22-release-gates
scripts/ai/verify_release.sh v22-strict-provenance
scripts/ai/verify_release_strict.sh v22
scripts/ai/verify_v22_release_gates.sh
scripts/ai/verify_v22_strict_provenance.sh
```

This release validates the workbench evidence and release provenance only. It
does not replace the later v0.23+ account/strategy/venue isolation work, v0.24+
operation-control semantics, or v0.25+ product-grade hardening.
