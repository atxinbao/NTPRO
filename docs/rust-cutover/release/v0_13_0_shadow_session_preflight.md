# NTPRO v0.13.0 Bounded Local Shadow Preflight Loop Contract

Date: 2026-06-21
Executor: Codex

## Positioning

`v0.13.0` remains Guarded Live Alpha Preflight only. The shadow preflight
surface is a bounded local evidence loop. It does not send production
orders and does not unlock Dashboard order controls.

Plain Chinese summary: 这一步只是让 shadow preflight 做成“本地有边界的预检循环”。
它会写心跳、能被 stop-file 停掉、能发现输入数据过期，但仍然不碰
生产下单、不撤单、不改订单。

## CLI

```text
nautilus live production-shadow-preflight-session \
  --run-id v130-shadow \
  --session-id v130-shadow-session \
  --strategy-id ema_cross_btcusdt_v1 \
  --shadow-portfolio-runtime shadow_portfolio_runtime.json \
  --strategy-session-status strategy_session_status.json \
  --output shadow_preflight_session.jsonl \
  --max-heartbeats 2 \
  --heartbeat-interval-ms 1000 \
  --stale-after-ms 30000 \
  --stop-file STOP
```

## Artifact

The command writes JSONL events with:

```text
schema_version = ntpro.v130_shadow_preflight_session_event.v1
event_type = shadow_preflight_session_started
event_type = shadow_preflight_session_heartbeat
event_type = shadow_preflight_session_stopped
event_type = shadow_preflight_stale_data_detected
```

Each event carries the no-mutation boundary:

```text
session_network_attempted = false
production_order_submissions_attempted = 0
production_orders_submitted = 0
production_order_mutations_attempted = 0
production_order_state_reads_attempted = 0
listen_key_lifecycle_attempted = 0
cancel_replace_amend_attempted = false
dashboard_order_controls_enabled = false
real_orders_submitted = false
values_are_exchange_truth = false
```

## Stop Handling

If the configured stop-file exists, the loop writes
`shadow_preflight_session_stopped` with:

```text
stop_file_observed = true
shutdown_reason = owner_stop_file
```

## Stale-Data Handling

If `shadow_portfolio_runtime.json` is older than `--stale-after-ms`, the loop
writes `shadow_preflight_stale_data_detected` with:

```text
state = stale_data_halted
stale_data_detected = true
shutdown_reason = stale_shadow_portfolio_runtime
```

## Non-Goals

This contract still excludes:

- production order submission;
- cancel / replace / amend / retry / correction;
- production order-state reads;
- listenKey lifecycle;
- production user-stream runtime;
- Dashboard order controls;
- automatic correction orders;
- real funds.

## Verification

```text
cargo test -p nautilus-cli production_shadow_preflight_session --lib
scripts/ai/verify_v13_shadow_preflight_session.sh
scripts/ai/verify_fast.sh
git diff --check
```
