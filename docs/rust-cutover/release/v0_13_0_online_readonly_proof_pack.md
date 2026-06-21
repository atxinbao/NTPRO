# NTPRO v0.13.0 Owner-run Online Read-only Proof Pack

Date: 2026-06-21
Executor: Codex
Milestone: `v0.13.0`
Status: SCOPE + LOCAL PREFLIGHT

## Summary

`v0.13.0` keeps production network proof owner-gated. The default proof-pack
command is offline and only verifies that production public/account read-only
paths fail closed unless the owner explicitly enables manual online mode.

Plain Chinese summary: 这份 proof pack 是“owner 手动证明包”，不是 CI 自动联网，
也不是实盘交易。默认运行只证明：没有 owner 明确开 gate 时，生产只读路径不会联网。
如果 owner 手动开 gate，proof pack 只允许生产 GET 只读：public time 和 account
snapshot。它仍然不下单、不撤单、不改单、不读生产订单状态、不创建 listenKey，也不给
Dashboard 增加下单按钮。

## Command

Default CI-safe preflight:

```bash
scripts/ai/verify_v13_online_readonly_proof_pack.sh
```

Optional owner-run online proof:

```bash
NTPRO_V13_OWNER_RUN_ONLINE_READONLY_PROOF=1 \
NTPRO_V13_OWNER_ACCEPTS_PRODUCTION_READONLY_RISK=1 \
NTPRO_V12_MANUAL_ONLINE=1 \
NTPRO_ALLOW_PRODUCTION_PUBLIC_READ=1 \
NTPRO_CONFIRM_PRODUCTION_PUBLIC_READ_ONLY=1 \
NTPRO_CONFIRM_NO_PRODUCTION_ORDER_MUTATION=1 \
NTPRO_ALLOW_PRODUCTION_AUTHENTICATED_READ=1 \
NTPRO_OWNER_APPROVED_PRODUCTION_READ_ONLY=1 \
NTPRO_CONFIRM_PRODUCTION_ACCOUNT_NO_ORDER_MUTATION=1 \
NTPRO_CONFIRM_NO_SECRET_PERSISTENCE=1 \
BINANCE_PRODUCTION_READONLY_API_KEY=... \
BINANCE_PRODUCTION_READONLY_API_SECRET=... \
scripts/ai/verify_v13_online_readonly_proof_pack.sh
```

## Manifest Contract

The proof pack writes:

```text
proof-pack-manifest.json
```

Required manifest markers:

```text
schema_version = ntpro.v130_online_readonly_proof_pack.v1
default_ci_network_required = false
owner_run_online_proof_required_for_release = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
production_order_state_reads_allowed = false
listen_key_lifecycle_allowed = false
dashboard_order_controls_enabled = false
real_funds_enabled = false
production_trading_enabled = false
artifacts_redacted = true
```

## Artifact Semantics

The proof pack can finish in two owner-run modes:

```text
owner_run_online_ok
owner_run_classified_failure
```

`owner_run_classified_failure` is still a redacted evidence package, but it is
not connectivity proof, not account snapshot proof, and not trading readiness.
The default preflight status is:

```text
offline_preflight_ok
```

## Redaction Requirements

Artifacts must not persist:

```text
raw account response
raw balances
raw permissions
API key value
API secret value
signature
signed query
signed URL
Dashboard credential input
```

## Explicit Non-Goals

This proof pack does not authorize production order submission, production
order mutation, production order-state reads, listenKey lifecycle, production
WebSocket user streams, automatic remediation, real funds, production trading,
or Dashboard order controls.
