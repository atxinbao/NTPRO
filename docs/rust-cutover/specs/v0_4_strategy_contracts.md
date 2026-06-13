# v0.4 Built-In Strategy Contracts - EMA and RSI

Date: 2026-06-13
Executor: Codex
Task ID: V04-003

## Scope

This document defines the built-in EMA and RSI strategy contracts for the
`v0.4.0` Binance Sandbox Product Foundation.

The contracts are product-facing and deterministic. They exist so later config,
fixture replay, mock execution, risk rejection, strategy smoke, and Dashboard
tasks can share one stable definition.

## Shared Contract

Both built-in strategies must stay inside this boundary:

- market: Binance Spot sandbox product boundary;
- data source: checked-in fixture replay;
- execution: mock order lifecycle only;
- risk: scoped sandbox risk checks only;
- order side: deterministic signal output, then mock order evidence;
- dashboard: read-only status derived from the sandbox run;
- credentials: no real API keys, accounts, balances, or live order
  acknowledgements.

Shared required input:

| Field | Type | Requirement |
| --- | --- | --- |
| `strategy_name` | string | Must be `ema` or `rsi`. |
| `instrument_id` | string | Must identify the sandbox Binance Spot instrument used by fixture replay. |
| `bar_type` | string | Must match the replay bar stream. |
| `trade_size` | decimal string | Must be positive and within sandbox risk limits. |
| `max_orders` | integer | Must be positive and bounded for deterministic smoke runs. |
| `risk_profile` | string | Must select a sandbox risk profile, not production risk. |

Shared output:

| Output | Requirement |
| --- | --- |
| `strategy_status` | One of `configured`, `running`, `completed`, or `rejected`. |
| `signals` | Deterministic signal list with timestamp, instrument, side, and reason. |
| `orders` | Mock lifecycle summary after `V04-008` exists. |
| `risk` | Accepted/rejected summary after `V04-009` exists. |
| `metrics` | Count of bars processed, signals emitted, orders submitted, fills, cancels, and rejections. |

Shared non-goals:

- no arbitrary user strategy loading;
- no production alpha or profitability claim;
- no live venue connectivity;
- no real order submission;
- no cross-exchange portability claim;
- no generic strategy registry.

## EMA Strategy Contract

Strategy name:

```text
ema
```

Plain language:

```text
EMA compares a fast exponential moving average with a slow exponential moving
average on replayed Binance Spot bars. A cross above may emit a buy signal; a
cross below may emit a sell or exit signal, depending on the configured mode.
```

Required EMA input:

| Field | Type | Requirement |
| --- | --- | --- |
| `fast_period` | integer | Required. Must be greater than 1. |
| `slow_period` | integer | Required. Must be greater than `fast_period`. |
| `signal_mode` | string | Required. `cross` for entry/exit cross events in v0.4. |
| `warmup_bars` | integer | Optional. Defaults to `slow_period`. Must be at least `slow_period`. |

EMA signal rules:

- Input bars are processed in fixture timestamp order.
- No signal may be emitted before warmup is complete.
- A fast EMA crossing above the slow EMA emits a deterministic `buy` signal.
- A fast EMA crossing below the slow EMA emits a deterministic `sell` or
  `exit` signal according to the later DTO mode.
- Equal EMA values do not emit a signal.

EMA expected evidence:

- config validation accepts valid EMA config and rejects invalid period order;
- fixture replay produces a stable bar count;
- EMA smoke records deterministic signal count and first/last signal summary;
- mock order lifecycle records how EMA signals map to submit/accept/fill/cancel
  or scoped reject outcomes;
- risk output is deterministic and includes rejection reason when applicable.

EMA non-goal:

- no claim that EMA is profitable;
- no dynamic parameter optimization;
- no production account execution;
- no support for arbitrary EMA variants beyond the v0.4 DTO.

## RSI Strategy Contract

Strategy name:

```text
rsi
```

Plain language:

```text
RSI measures momentum on replayed Binance Spot bars. In NTPRO the RSI value is
treated as a normalized value in the range 0.0 to 1.0 for v0.4 contracts. A
value below the oversold threshold may emit a buy signal; a value above the
overbought threshold may emit a sell or exit signal.
```

Required RSI input:

| Field | Type | Requirement |
| --- | --- | --- |
| `period` | integer | Required. Must be greater than 1. |
| `oversold_threshold` | decimal string | Required. Must be greater than or equal to `0.0` and less than `overbought_threshold`. |
| `overbought_threshold` | decimal string | Required. Must be less than or equal to `1.0` and greater than `oversold_threshold`. |
| `warmup_bars` | integer | Optional. Defaults to `period`. Must be at least `period`. |

RSI signal rules:

- Input bars are processed in fixture timestamp order.
- No signal may be emitted before warmup is complete.
- RSI below `oversold_threshold` emits a deterministic `buy` signal only when
  the strategy is flat or otherwise allowed by sandbox risk.
- RSI above `overbought_threshold` emits a deterministic `sell` or `exit`
  signal according to the later DTO mode.
- RSI values inside the threshold band do not emit entry signals.

RSI expected evidence:

- config validation accepts valid RSI thresholds and rejects inverted thresholds;
- fixture replay produces a stable bar count;
- RSI smoke records deterministic signal count and first/last signal summary;
- mock order lifecycle records how RSI signals map to submit/accept/fill/cancel
  or scoped reject outcomes;
- risk output is deterministic and includes rejection reason when applicable.

RSI non-goal:

- no claim that RSI is profitable;
- no dynamic threshold optimization;
- no production account execution;
- no support for arbitrary oscillator variants beyond the v0.4 DTO.

## Evidence Model

Later V04 smokes should write comparable evidence for EMA and RSI:

| Evidence field | Requirement |
| --- | --- |
| `strategy_name` | `ema` or `rsi`. |
| `fixture_id` | Name or path of checked-in fixture set. |
| `instrument_id` | Sandbox Binance Spot instrument. |
| `bars_processed` | Deterministic count. |
| `signals_emitted` | Deterministic count. |
| `orders_submitted` | Deterministic mock count after `V04-008`. |
| `orders_filled` | Deterministic mock count after `V04-008`. |
| `orders_cancelled` | Deterministic mock count after `V04-008`. |
| `risk_rejections` | Deterministic count and reason after `V04-009`. |
| `production_boundary` | Must state `no_real_funds`, `no_real_orders`, and `sandbox_only`. |

## Compatibility With Later Tasks

- `V04-004` must encode these fields into the stable strategy config DTO.
- `V04-005` must provide replay data usable by both contracts.
- `V04-008` must provide order lifecycle output consumed by both smokes.
- `V04-009` must provide risk rejection output consumed by both smokes.
- `V04-006` must implement the EMA smoke according to this contract.
- `V04-007` must implement the RSI smoke according to this contract.
- `V04-010` must display strategy state from this evidence model, not
  placeholder values.
