# NTPRO v0.13.1 Money Price Quantity Contract Draft

Date: 2026-06-22
Executor: Codex
Milestone: `v0.13.1`
Task: `V131-005`
Status: contract draft

## Summary

V131-005 defines the Money/Price/Quantity contract that future live-alpha
preflight must satisfy before it can claim risk/execution-grade money math.
This is a release-facing contract only. It does not implement production money
math and does not authorize any production order path.

Plain Chinese summary: v0.13.1 这里只是在写“以后实盘金额、价格、数量怎么算才算合格”
的合同。现在仍然不是实盘 money math 完成，不是风控/执行级计算完成，也没有打开下单、
撤单、改单、订单状态读取、listenKey、真实资金或 Dashboard 下单控件。

## Relationship To v0.13.0 Decimal Boundary

`v0_13_0_decimal_amount_boundary.md` fixed the representation boundary:
amount-like fields must be plain decimal strings and must not use `f64`
aggregation.

This v0.13.1 contract draft adds the missing product contract for future
venue/account/instrument preflight:

- which precision values must be known;
- which exchange filters must be known;
- which rounding mode must be recorded;
- which min/max quantity, price, and notional limits must be checked;
- which fee and slippage inputs must be present before execution-grade claims.

## Representation Contract

All Money, Price, and Quantity values must be transported as plain decimal
strings:

```text
decimal_string_required=true
negative_values=false
scientific_notation=false
nan_or_infinity=false
binary_float_transport=false
f64_aggregation_used=false
parser=rust_decimal
```

The following values are in scope for the future contract:

```text
price
quantity
notional
fee_amount
slippage_amount
max_order_notional
max_position_notional
max_daily_notional
remaining_risk_budget
min_order_quantity
max_order_quantity
```

## Required Instrument And Venue Fields

Future live-alpha preflight must have these identity fields before money math
can be considered risk/execution-grade:

```text
instrument_id
venue
base_asset
quote_asset
settlement_asset
account_id
strategy_id
```

These fields must be treated as evidence inputs. v0.13.1 does not claim the
values are exchange truth.

```text
source=contract_draft_only
values_are_exchange_truth=false
```

## Required Precision Fields

Future live-alpha preflight must record precision explicitly:

```text
asset_precision
quote_precision
price_precision
quantity_precision
notional_precision
```

Plain Chinese summary: 不能只拿一个字符串价格就直接算。必须知道这个交易对、资产、
报价资产分别允许几位小数，否则无法判断订单能不能被交易所接受。

## Required Exchange Filter Fields

Future live-alpha preflight must record exchange filter values explicitly:

```text
tick_size
step_size
min_price
max_price
min_quantity
max_quantity
min_notional
max_notional
max_order_notional
max_position_notional
max_daily_notional
```

These values define whether a candidate order is admissible after precision
normalization and rounding.

## Rounding Contract

Future live-alpha preflight must record rounding mode explicitly for each value
type:

```text
rounding_mode_price
rounding_mode_quantity
rounding_mode_notional
```

Required future behavior:

- price must be normalized against `tick_size` before admissibility checks;
- quantity must be normalized against `step_size` before admissibility checks;
- notional must be recomputed from normalized price and quantity;
- `min_notional` and `max_notional` must be checked after rounding;
- hidden floor/ceil behavior is not allowed;
- every rejection must record whether it came from precision, tick size, step
  size, quantity limit, price limit, notional limit, fee, slippage, or risk
  budget.

v0.13.1 does not implement these checks. It only records the contract future
implementations must satisfy.

## Fee And Slippage Inputs

Future execution-grade preflight must include fee and slippage inputs:

```text
fee_rate
fee_asset
fee_amount
slippage_limit_bps
slippage_amount
```

If fee or slippage inputs are absent, the future preflight must either reject
the order candidate or mark the evidence as incomplete. v0.13.1 does not
implement fee or slippage money math.

## Boundary Markers

```text
money_price_quantity_contract=draft_only
live_alpha_money_math_ready=false
risk_or_execution_grade=false
production_order_submission_allowed=false
production_order_mutation_allowed=false
production_order_state_reads_allowed=false
listen_key_lifecycle_allowed=false
dashboard_order_controls_enabled=false
real_orders_submitted=false
production_trading_enabled=false
values_are_exchange_truth=false
```

## Non-Claims

This contract draft does not implement production live-alpha money math. It
does not authorize any risk, execution, adapter, strategy, node, supervisor,
CLI, or Dashboard path to submit, cancel, replace, amend, retry, correct, read
production order state, create listenKey sessions, operate real funds, or
otherwise mutate production exchange orders.

## Future Acceptance Before Capability Claim

Before NTPRO can claim risk/execution-grade live-alpha money math, future work
must add evidence for:

- multiple assets with different precision and quote precision;
- tick-size rounding and rejection tests;
- step-size rounding and rejection tests;
- min/max quantity tests;
- min/max notional tests after rounding;
- fee and slippage handling or explicit fail-closed rejection;
- risk budget exhaustion and rejection evidence;
- golden trace coverage before any risk/execution production path uses the
  contract.
