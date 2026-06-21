# NTPRO v0.13.0 No-Production-Mutation Release Gate

Date: 2026-06-21
Executor: Codex
Milestone: `v0.13.0`
Task: `V130-007`
Status: release gate contract

## Summary

V130-007 wires the v0.13 Guarded Live Alpha Preflight evidence into release and
PR gates. The gate proves that default local, PR, and tag-release execution
remains offline/fail-closed and does not mutate production exchange state.

Plain Chinese summary: v0.13 可以收集 live alpha 前置证据，但发版门禁必须证明它
没有开始实盘交易。默认 gate 不联网做生产 mutation，不下单、不撤单、不改单、不读生产
订单状态、不创建 listenKey、不启用 Dashboard 下单控件、不声明实盘 money math 完成。

## Gate Stage

```bash
scripts/ai/verify_v13_no_production_mutation_gate.sh
scripts/ai/verify_release.sh v13-no-production-mutation-gate
```

## Gate Composition

The v13 no-production-mutation gate runs:

```text
verify_v13_shadow_preflight_session.sh
verify_v13_online_readonly_proof_pack.sh
verify_v13_kill_switch_approval_artifact.sh
verify_v13_dashboard_control_boundary.sh
verify_v13_decimal_amount_boundary.sh
```

The online proof pack is run in default offline preflight mode. Owner-run online
proof remains separately gated by explicit environment variables and is not
required by CI or release tag execution.

## Required Boundary Markers

```text
network_default_offline=true
production_order_submission_allowed=false
production_order_mutation_allowed=false
production_order_state_reads_allowed=false
listen_key_lifecycle_allowed=false
dashboard_order_controls_enabled=false
production_reconnect_allowed=false
live_alpha_money_math_ready=false
risk_or_execution_grade=false
real_orders_submitted=false
production_trading_enabled=false
```

## Release Integration

- `scripts/ai/verify_release.sh all` includes
  `v13-no-production-mutation-gate`.
- `scripts/ai/verify_release.sh v13-no-production-mutation-gate` is callable
  as a standalone stage.
- `.github/workflows/release-tag.yml` includes a
  `release-v13-no-production-mutation-gate` stage.
- `.github/workflows/rust-cutover-smoke.yml` classifies v0.13 release docs,
  v13 scripts, and workflow changes so the PR smoke runs the v13 gate.

## Non-Claims

This gate does not authorize production order mutation. It does not implement a
production live alpha. It does not create production order, cancel, replace,
amend, retry, correction, reconnect, order-state read, listenKey, credential
entry, real funds, or production trading capability.
