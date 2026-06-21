# NTPRO v0.13.0 Decimal Amount Boundary

Date: 2026-06-21
Executor: Codex
Milestone: `v0.13.0`
Task: `V130-006`
Status: boundary contract

## Summary

V130-006 defines the Decimal/string-only amount boundary for Guarded Live Alpha
Preflight. v0.13 may record evidence that local shadow notionals are parsed and
summed as Decimal strings, but it must not claim live-alpha risk/execution-grade
money math readiness.

Plain Chinese summary: v0.13 只把“金额字段必须怎么表示”定死。以后如果要进入
live alpha，price、quantity、notional、limit 这类字段必须是普通十进制字符串，并用
`Decimal` 处理。v0.13 仍然不是实盘金额计算完成，不是风险/执行级 money math 完成。

## Required Amount Representation

Future live alpha risk/execution preflight fields must use plain decimal
strings for amount-like values:

```text
price
quantity
notional
max_order_notional
max_position_notional
max_daily_notional
remaining_risk_budget
min_order_quantity
max_order_quantity
```

Plain decimal string means:

```text
allowed_characters=0123456789.
negative_values=false
scientific_notation=false
nan_or_infinity=false
binary_float_transport=false
```

## Boundary Markers

```text
amount_boundary=decimal_string_only
parser=rust_decimal
aggregation=rust_decimal_string_sum
f64_aggregation_used=false
live_alpha_money_math_ready=false
risk_or_execution_grade=false
scientific_notation_allowed=false
production_order_submission_allowed=false
production_order_mutation_allowed=false
real_orders_submitted=false
production_trading_enabled=false
```

## Existing Evidence Source

The v0.12/v0.12.1 shadow portfolio runtime already records:

```text
notional_preflight.status=shadow_decimal_string_evidence_only
notional_preflight.aggregation=rust_decimal_string_sum
notional_preflight.f64_aggregation_used=false
notional_preflight.live_alpha_money_math_ready=false
notional_preflight.risk_or_execution_grade=false
```

V130-006 keeps that evidence path, tightens the parser to reject scientific
notation, and documents that this remains evidence only.

## Non-Claims

This boundary does not implement production live-alpha money math. It does not
authorize any risk, execution, adapter, strategy, node, supervisor, CLI, or
Dashboard path to submit, cancel, replace, amend, retry, correct, or otherwise
mutate production exchange orders.
