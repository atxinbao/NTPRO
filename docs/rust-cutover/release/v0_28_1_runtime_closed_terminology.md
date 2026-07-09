# v0.28.1 Runtime-Closed Terminology Contract

Date: 2026-07-09
Executor: Codex
Task: `V281-005` / GitHub issue `#923`
Milestone: `v0.28.1`

## Goal

Harden the `runtime-closed` vocabulary so v0.28.0 deterministic artifact
evidence cannot be mistaken for live backend service integration or production
execution readiness.

Plain Chinese summary: `runtime-closed` 在 v0.28.x 中只表示确定性的 artifact replay
closure。它不是 live IdP/SSO、真实部署执行、adapter send、production trading runtime，
也不是 product-ready live trading terminal。

## Terms

```text
deterministic_artifact_replay_closure = source-controlled artifacts plus local deterministic replay and release-gate validation
backend_service_runtime = running backend service/process/API integration that owns live operational state
live_external_integration = real external provider, IdP/SSO, deployment platform, adapter, exchange, or network dependency
production_execution_runtime = production environment where submit, mutation, adapter send, exchange request, or trading controls can affect live state
runtime_closed_terminology = deterministic_artifact_replay_closure_only
```

## v0.28.x Meaning

`runtime-closed` remains the readiness-matrix classification value, but its
meaning in v0.28.x is exactly `deterministic_artifact_replay_closure`.

Every `runtime-closed` matrix entry must carry:

```text
closure_mode = deterministic_artifact_replay
backend_service_runtime_claim_allowed = false
live_external_integration_claim_allowed = false
production_execution_runtime_claim_allowed = false
product_ready_claim_allowed = false
```

## Forbidden Positive Claims

Deterministic artifacts and replay evidence must fail release gates if they are
described as:

```text
live external IdP
live deployment execution
live adapter send
production trading runtime
product-ready live trading
backend service runtime integration
```

This does not weaken the v0.28.0 backend closure value. It preserves the closed
artifact/replay evidence line and blocks only user-facing wording that would
imply a live service or production execution boundary.

## v0.29.0 Vocabulary Boundary

The next capability track may build on v0.28.x backend closure only if it keeps
these vocabulary boundaries:

```text
artifact replay closure may support planning for service integration
artifact replay closure may not be renamed to service integration
live external integration requires live or sandbox service evidence
production execution runtime requires explicit submit/mutation/send boundary evidence
product-grade terminal readiness requires separate frontend and operation-control evidence
```
