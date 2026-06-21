# v0.12.0 Offline and Owner-Gated Online Release Gates

Date: 2026-06-21
Executor: Codex
Task: V120-008

## Positioning

v0.12 release verification has two paths:

```text
offline release gate        = required, CI-safe, no production network
manual online preflight     = required as dry run, proves owner gates fail closed
real online production read = optional owner-run successful proof artifact, not required by CI
```

Plain Chinese summary: 默认发版只证明“代码、artifact、边界都正确，而且不会偷偷联网”。
真正的生产只读联网证明需要 owner 手动提供环境变量和只读凭证，不是自动 CI 的一部分。
如果 owner 不运行真实在线证明，v0.12.0 仍然可以发布，但 release notes/readiness 只能
声明“owner-gated 路径已实现、默认 gate fail-closed 已通过”，不能声明已经完成生产在线
成功证明。

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

## Owner-Run Evidence Artifact Contract

When an owner chooses to run the optional online proof, the result must be
recorded as an owner evidence artifact, not as an automatic release-gate
requirement.

Required public read artifact:

```text
path = <owner evidence root>/command-output/manual-online-public-read-probe.json
schema_version = ntpro.v120_production_public_online_read_probe.v1
status = online_read_probe_ok
method = GET
path = /api/v3/time
network_attempted = true
production_public_online_read_attempted = true
online_read_allowed = true
response_shape = binance_server_time_v1
response_shape_validated = true
credentials_used = false
production_order_submission_attempted = false
production_order_mutation_attempted = false
dashboard_order_controls_enabled = false
```

Required authenticated account artifact:

```text
path = <owner evidence root>/command-output/manual-online-account-snapshot.json
schema_version = ntpro.v120_authenticated_account_snapshot_online_read.v1
status = online_account_snapshot_ok
method = GET
path = /api/v3/account
network_attempted = true
account_read_attempted = true
online_read_allowed = true
response_shape = binance_account_snapshot_v1
response_shape_validated = true
signature_recorded = false
signed_query_recorded = false
signed_url_recorded = false
api_key_value_recorded = false
api_secret_value_recorded = false
production_order_submission_attempted = false
production_order_mutation_attempted = false
dashboard_order_controls_enabled = false
```

Classified failures such as timeout, connect error, non-success HTTP status, or
response-shape mismatch are useful diagnostics, but they are not successful
online proof artifacts.

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

- owner-run successful production online proof by default;
- production trading readiness;
- production order submission support;
- production order-state read support;
- automatic production remediation;
- Dashboard order controls;
- real-funds readiness.
