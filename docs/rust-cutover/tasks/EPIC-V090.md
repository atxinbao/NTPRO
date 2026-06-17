# EPIC-V090 Strategy Runtime Foundation

Date: 2026-06-18
Executor: Codex
Owner role: Control & Scope Agent
Review role: Verification & Release Gatekeeper
Risk level: high
Status: BACKLOG

## Goal

Move NTPRO from workflow proof toward a headless strategy runtime product:
`ntpro-node` can host a strategy session, consume fixture/mock market input,
write signal/order-intent/risk-decision/audit artifacts, and expose read-only
supervisor/dashboard status.

## Boundary

Included:

- strategy session config and lifecycle;
- fixture/mock market stream;
- built-in demo strategy runtime;
- signal artifact;
- order intent artifact;
- shadow-mode risk decision artifact;
- strategy session audit and summary;
- `ntpro-node` strategy host;
- supervisor status;
- Dashboard read-only display;
- v0.9 smoke/release wiring;
- v0.9 readiness and release-note material.

Excluded:

- Binance testnet order submission;
- production order submission;
- real funds;
- production trading;
- Dashboard order controls;
- strategy-driven live exchange execution.

## Version Decision

```text
v0.9.0  = Strategy Runtime Foundation
v0.10.0 = Binance Testnet Order Proof
v0.11.0 = Production Read-Only + Shadow
v0.12.0 = Guarded Live Alpha
```

`v0.9.0` must stop at `order_intent + risk_rejection`. Testnet order proof is
deferred to `v0.10.0`.

## Automation Policy

`V090-000` through `V090-013` are scoped as medium-risk, offline-first
automation slices. They may use auto-merge after local validation and hosted
smoke checks pass.

`V090-014` remains a release-closure task. It must not create a tag or publish a
GitHub Release until the owner gives explicit release approval.
