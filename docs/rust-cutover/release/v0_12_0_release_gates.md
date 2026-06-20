# v0.12.0 Offline and Owner-Gated Online Release Gates

Date: 2026-06-21
Executor: Codex
Task: V120-008

## Positioning

v0.12 release verification has two paths:

```text
offline release gate        = required, CI-safe, no production network
manual online preflight     = required as dry run, proves owner gates fail closed
real online production read = optional owner-run proof, not required by CI
```

Plain Chinese summary: 默认发版只证明“代码、artifact、边界都正确，而且不会偷偷联网”。
真正的生产只读联网证明需要 owner 手动提供环境变量和只读凭证，不是自动 CI 的一部分。

## Offline Gate

The required offline gate is:

```bash
scripts/ai/verify_v12_offline_release_gates.sh
scripts/ai/verify_release.sh v12-offline-release-gates
```

It verifies:

- public production read-only probe fails closed by default;
- authenticated account snapshot preflight fails closed by default;
- response-shape artifacts stay redacted;
- shadow portfolio runtime is local and non-mutating;
- shadow strategy session is persistent and non-mutating;
- read-only reconciliation never submits or corrects production orders;
- Dashboard production shadow panel remains read-only.

## Manual Online Preflight

The required dry-run preflight is:

```bash
scripts/ai/verify_v12_manual_online_preflight.sh
scripts/ai/verify_release.sh v12-manual-online-preflight
```

It proves that manual-online requests still stop before network when
`NTPRO_V12_MANUAL_ONLINE=1` is not set.

## Optional Owner Online Proof

Optional real production read-only proof remains outside default CI. Owners may
run the underlying v12 scripts with all required gates and read-only
credentials:

```bash
NTPRO_V12_MANUAL_ONLINE=1 \
NTPRO_ALLOW_PRODUCTION_PUBLIC_READ=1 \
NTPRO_CONFIRM_PRODUCTION_PUBLIC_READ_ONLY=1 \
NTPRO_CONFIRM_NO_PRODUCTION_ORDER_MUTATION=1 \
scripts/ai/verify_v12_public_online_read_probe.sh
```

Authenticated account read proof additionally requires read-only credentials
and the authenticated owner gates documented in
`scripts/ai/verify_v12_authenticated_account_snapshot_online_read.sh`.

## Boundaries

The v0.12 gates must keep these invariants:

```text
production_orders_submitted=0
production_order_mutations_attempted=0
production_order_state_reads_attempted=0
listen_key_lifecycle_attempted=0
automatic_correction_orders_submitted=0
dashboard_order_controls_enabled=false
real_orders_submitted=false
```

## Non-Claims

These gates do not claim:

- production trading readiness;
- production order submission support;
- production order-state read support;
- automatic production remediation;
- Dashboard order controls;
- real-funds readiness.
