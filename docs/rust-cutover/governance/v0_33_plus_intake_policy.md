# v0.33.0+ Separately Scoped Intake Policy

Date: 2026-07-15
Executor: Codex
Backend baseline: `ntpro-rust-only-v0.32.0`
Status: ACTIVE AFTER BFG-006 MERGE

## Purpose

This policy controls entry into any v0.33.0+ capability track after the v0.32.0
backend freeze. It authorizes intake design only. It does not create a version,
milestone, implementation issue, release, or production capability.

Plain Chinese summary: v0.33.0+ 不是后端主线续版。先单独定义产品目标、owner、模块
边界、依赖、数据契约、回滚、telemetry 和禁止能力，再决定是否建立 milestone。
v0.32.0 只能作为只读后端基线，不能自动授权 submit、mutation、adapter send、live
exchange、retry、remediation、recovery、交易控件或实际 backend go-live。

## Default Eligible Directions

- frontend Trader Terminal presentation and read-only workflows;
- trader UX and operator-facing product workflows without production execution
  authority;
- product deployment, upgrade, diagnostics, and operating experience that does
  not expand the frozen backend boundary;
- a new module with independent ownership, contracts, validation, and release
  scope.

Eligibility does not imply approval. Each proposal must complete the contract
below before GitHub milestone materialization.

## Mandatory Intake Contract

Every proposal must provide all fields. `TBD`, inherited-from-v0.32.0, and an
empty field are fail-closed values.

### Identity and Ownership

- proposed track and task prefix;
- product outcome and target users;
- accountable owner and separate review owner;
- risk level and backend-freeze classification.

### Scope and Boundary

- included modules and allowed paths;
- prohibited paths and non-goals;
- explicit frozen registry fields touched, normally none;
- statement that v0.32.0 is a dependency baseline, not capability authority;
- exception issue and authorization when any frozen field is touched.

### Dependencies

- exact prerequisite issues, PRs, contracts, and gates;
- dependency ordering encoded in milestone, issue body, and comments;
- no circular dependency on a release that the proposal itself is meant to
  create.

### Data and Interface Contract

- input and output schemas, ownership, freshness, and provenance;
- read-only versus mutating interface classification;
- error, stale, missing, partial, and incompatible-data behavior;
- public API and migration impact;
- confirmation that UI intent cannot become backend execution authority by
  implication.

### Operation, Rollback, and Telemetry

- deployment and configuration boundary;
- deterministic rollback or disable path;
- telemetry, SLO, alert, and incident ownership;
- audit retention and reconstruction path;
- expected resource and performance limits.

### Validation

- unit, contract, smoke, and relevant golden replay coverage;
- backend-freeze baseline gate;
- negative tests for every forbidden or missing-boundary state;
- release and publication requirements when a release is actually proposed.

## Forbidden Default Inheritance

The proposal must explicitly set every item below to false unless an approved
`backend-freeze-exception` issue provides separate authorization:

- backend go-live and actual production go-live;
- production submit and order mutation;
- cancel, replace, amend, and flatten authority;
- execution adapter call/send and live exchange request;
- implicit retry, retry scheduler, automatic remediation, automatic operation,
  and automatic recovery;
- Dashboard and Admin Workbench operation or trading controls;
- Trader Terminal order ticket or manual submit authority;
- frontend completion and product-grade live trading terminal claims;
- default or strategy-driven production execution.

## GitHub Materialization Gate

A v0.33.0+ milestone may be created only after one approved intake issue proves:

1. the mandatory contract is complete;
2. `backend_freeze_classification=v33-separately-scoped`;
3. `forbidden_capability_request=none` or a separately approved exception;
4. `scripts/ai/verify_release.sh backend-freeze-baseline` passes;
5. milestone and issue bodies repeat the non-inheritance statement;
6. each dependency has a GitHub-visible comment using actual issue numbers.

The milestone title, issue count, and delivery schedule are intentionally not
defined by this policy. They require a later, separately authorized planning
decision.

## Rejection Conditions

- a proposal describes itself as continuation of the backend mainline;
- owner, review owner, rollback, telemetry, or data contract is missing;
- production authority is inferred from read-only v0.32.0 evidence;
- the proposal hides a forbidden capability behind UI, adapter, retry, or
  recovery wording;
- a milestone or implementation issue is created before intake approval.

Rejected proposals remain unmaterialized. They do not create a default v0.32.1
or v0.33.0 release obligation.
