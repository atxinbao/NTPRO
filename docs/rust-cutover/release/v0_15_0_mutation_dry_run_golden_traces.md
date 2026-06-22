# NTPRO v0.15.0 Mutation Dry-run Golden Traces

Date: 2026-06-22
Executor: Codex
Milestone: `v0.15.0`
Task: `V150-006`
Status: implementation evidence pending PR closeout

## Summary

`v0.15.0` adds executable golden traces for the guarded production live-alpha
mutation dry-run chain. The traces prove rejection and preflight states stay
local and cannot submit production orders, attempt production mutation, open
network access, or call a production execution adapter.

Plain Chinese summary: 这不是开放实盘交易。它只是把 v0.15 干跑链路的关键挡板变成
可回放证据：kill switch、审批、风控、本地网络禁用、Dashboard 控制禁用都要能挡住，
并且每条 trace 都证明真实生产 mutation 没发生。

## Product Boundary

Included:

- Nine executable golden trace cases for mutation dry-run rejection/preflight
  states.
- CLI replay harness:
  `cargo test -p nautilus-cli --test golden_trace_live_alpha_mutation_dry_run`.
- Release gate stage:
  `scripts/ai/verify_full.sh golden-traces-live-alpha-mutation-dry-run`.
- Release replay scope entries for every new trace case.

Not included:

- Production order submission.
- Production cancel, replace, amend, retry, correction, or automatic
  remediation.
- Production HTTP request execution.
- Production execution adapter call.
- listenKey lifecycle.
- Real funds.
- Production trading.
- Dashboard order controls.

## Evidence Contract

Each replayed event must prove:

```text
production_orders_submitted=0
production_order_mutations_attempted=0
network_attempted=false
execution_adapter_called=false
```

The `network_disabled` case may reach the local dry-run execution adapter
artifact, but it still records:

```text
dry_run_execution_adapter_called=true
production_adapter_called=false
network_attempted=false
```

## Validation

Expected local validation:

```text
python3 scripts/ai/golden_trace_runner.py tests/golden/live_alpha_mutation_dry_run_schema.jsonl --mode validate-only
cargo test -p nautilus-cli --test golden_trace_live_alpha_mutation_dry_run
scripts/ai/verify_full.sh golden-traces-files
scripts/ai/verify_full.sh golden-traces-live-alpha-mutation-dry-run
```
