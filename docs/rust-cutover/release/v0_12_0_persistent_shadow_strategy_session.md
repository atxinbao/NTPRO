# v0.12.0 Bounded Shadow Strategy Session Event Artifact

Date: 2026-06-21
Executor: Codex
Task: V120-005

## Positioning

V120-005 adds a local, read-only, bounded shadow strategy session event artifact
for v0.12.0. It records JSONL start, heartbeat, artifact-gap, and stop events
from already-generated shadow artifact evidence.

This is not a long-running strategy runtime, daemon, heartbeat loop, or stale
data monitor. Those runtime semantics are reserved for a later
owner-approved v0.13 preflight scope.

Plain Chinese summary: 这个能力只是生成一份本地 JSONL 事件文件。它告诉用户：
shadow 策略会话记录里有 start、有限个 heartbeat、artifact 缺口和 stop。它不是后台常驻
进程，不会持续跑策略，也不代表策略已经能真实下单，更不代表 shadow 状态就是交易所确认状态。

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

`--heartbeat-count` bounds the number of heartbeat events written. The command
terminates after writing the requested finite event set; it does not keep a
process alive for supervision.

## Artifact

```text
schema_version=ntpro.v120_shadow_strategy_session_event.v1
artifact=v0_12/shadow_strategy_session.jsonl
```

Each event records:

- run/session/strategy identity;
- bounded event type: start, heartbeat, artifact gap, or stop;
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
