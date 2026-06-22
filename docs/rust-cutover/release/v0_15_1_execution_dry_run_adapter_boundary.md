# v0.15.1 Execution Dry-Run Adapter Boundary Contract

Date: 2026-06-23
Executor: Codex
Milestone: v0.15.1
Status: release-facing contract

## Summary

This document defines the v0.15.1 production live-alpha execution dry-run
adapter boundary. It does not add production execution. It makes the local
dry-run route auditable and keeps the production execution adapter outside the
allowed path.

Plain Chinese summary: v0.15.1 这里要证明的不是“能实盘下单”，而是“策略意图
经过风控后，只能生成本地 dry-run 执行命令，最后只写本地 dry-run adapter 证据；
生产 adapter 不能被路由、不能被实例化、不能被调用”。

## Boundary Chain

```text
StrategyIntent
  -> RiskDecision
  -> ExecutionCommand
  -> DryRunExecutionAdapter
```

The only v0.15.1 execution path is the local dry-run artifact path:

```text
execution_command_route = dry_run_adapter_only
execution_command_destination = ntpro_local_artifact_dry_run_execution_adapter
```

The production adapter path remains forbidden:

```text
production_adapter_route_allowed = false
production_adapter_instantiation_allowed = false
production_adapter_called = false
```

## Artifact Contract

The execution dry-run artifact keeps the v0.15 schema:

```text
schema_version = ntpro.v150_live_alpha_execution_dry_run.v1
```

v0.15.1 adds an explicit boundary contract version inside that artifact:

```text
execution_boundary_contract_version = ntpro.v151_execution_dry_run_adapter_boundary.v1
execution_boundary_flow = StrategyIntent -> RiskDecision -> ExecutionCommand -> DryRunExecutionAdapter
```

Ready path requirements:

```text
execution_boundary_contract_ready = true
strategy_intent_boundary = StrategyIntent
risk_decision_boundary = RiskDecision
execution_command_boundary = ExecutionCommand
execution_command_created = true
execution_command_route = dry_run_adapter_only
execution_command_destination = ntpro_local_artifact_dry_run_execution_adapter
dry_run_adapter_boundary = DryRunExecutionAdapter
dry_run_adapter_route_allowed = true
production_adapter_boundary = ProductionExecutionAdapter
production_adapter_route_allowed = false
production_adapter_instantiation_allowed = false
dry_run_execution_adapter_called = true
dry_run_execution_adapter_wrote_artifact = true
production_adapter_called = false
network_attempted = false
production_orders_submitted = 0
production_order_mutations_attempted = 0
dashboard_order_controls_enabled = false
```

Blocked path requirements:

```text
execution_boundary_contract_ready = false
execution_command_created = false
execution_command_route = blocked_before_execution_command
execution_command_destination = none
dry_run_adapter_route_allowed = false
production_adapter_route_allowed = false
production_adapter_instantiation_allowed = false
dry_run_execution_adapter_called = false
production_adapter_called = false
network_attempted = false
production_orders_submitted = 0
production_order_mutations_attempted = 0
dashboard_order_controls_enabled = false
```

## Not Included

```text
ProductionExecutionAdapter implementation
ProductionExecutionAdapter instantiation
production order submission
production order mutation
production HTTP request execution
cancel / replace / amend / retry / correction
listenKey lifecycle
real exchange state
real funds
production trading
Dashboard order controls
```

## Validation

The boundary is enforced by:

```text
cargo test -p nautilus-cli production_live_alpha_execution_dry_run --lib
scripts/ai/verify_v15_execution_adapter_isolation.sh
scripts/ai/verify_release.sh v15-release-gates
```

The script checks both blocked and ready artifact paths and fails if the local
dry-run path leaks into a production adapter, network call, or production order
mutation.
