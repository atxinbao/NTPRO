# v0.12.0 Persistent Shadow Strategy Session

Date: 2026-06-21
Executor: Codex
Task: V120-005

## Positioning

V120-005 adds a local, read-only persistent shadow strategy session artifact for
v0.12.0. It records a session event stream from already-generated shadow
runtime evidence.

Plain Chinese summary: 这个能力是给生产只读 shadow runtime 做“会话状态记录”。它告诉用户：
shadow 策略会话启动了、心跳还在、输入 artifact 有没有缺口、本地是否停止。它不代表策略
已经能真实下单，也不代表 shadow 状态就是交易所确认状态。

## CLI

```bash
nautilus live production-shadow-strategy-session \
  --run-id v120-shadow \
  --session-id v120-shadow-session \
  --strategy-id ema_cross_btcusdt_v1 \
  --shadow-portfolio-runtime v0_12/shadow_portfolio_runtime.json \
  --strategy-session-status strategy/session_status.json \
  --output v0_12/shadow_strategy_session.jsonl \
  --heartbeat-count 2 \
  --stop-after-heartbeats
```

`--strategy-session-status` is optional. When it is missing or omitted, the
command records an owner-visible artifact-gap event and continues with degraded
local read-only evidence.

## Artifact

```text
schema_version=ntpro.v120_shadow_strategy_session_event.v1
artifact=v0_12/shadow_strategy_session.jsonl
```

Each event records:

- run/session/strategy identity;
- event type: start, heartbeat, artifact gap, or stop;
- shadow portfolio runtime reference;
- optional strategy session status reference;
- artifact-gap reason when present;
- production order submission and mutation counters fixed at `0`;
- Dashboard order controls fixed at `false`;
- `values_are_exchange_truth=false`.

## Boundary

The command rejects shadow portfolio runtime inputs when they claim or record:

- production order submission;
- production order mutation;
- automatic correction orders;
- real orders;
- Dashboard order controls;
- full production portfolio parity;
- exchange-truth values.

## Verification

```bash
cargo test -p nautilus-cli production_shadow_strategy_session --lib
scripts/ai/verify_v12_persistent_shadow_strategy_session.sh
scripts/ai/verify_v12_shadow_portfolio_runtime.sh
scripts/ai/verify_fast.sh
git diff --check
```
