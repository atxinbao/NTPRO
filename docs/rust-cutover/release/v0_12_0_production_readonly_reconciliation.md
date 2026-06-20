# v0.12.0 Production Read-Only Reconciliation

Date: 2026-06-21
Executor: Codex
Task: V120-006

## Positioning

V120-006 turns the earlier reconciliation event model into a local v0.12 CLI
artifact. It classifies local shadow evidence and records what an owner should
review next.

Plain Chinese summary: 这个能力只是在本地判断“shadow 证据现在是什么状态”。它不会自动
纠错，不会去交易所查订单状态，不会撤单、改单、补单，也不会把不一致自动变成交易动作。

## CLI

```bash
nautilus live production-readonly-reconciliation \
  --run-id v120-shadow \
  --account-snapshot v0_12/production_account_snapshot_redacted.json \
  --shadow-portfolio-runtime v0_12/shadow_portfolio_runtime.json \
  --shadow-strategy-session v0_12/shadow_strategy_session.jsonl \
  --shadow-intent v0_12/shadow_execution_intent.jsonl \
  --output v0_12/reconciliation_events.jsonl
```

All inputs are local artifacts. Missing optional inputs are classified into
read-only reconciliation states instead of triggering production activity.

## Artifact

```text
schema_version=ntpro.v120_readonly_reconciliation_event.v1
artifact=v0_12/reconciliation_events.jsonl
```

Each event records:

- run identity;
- reconciliation classification;
- severity and recommended local-only action;
- local artifact references;
- risk-halt/manual-review booleans;
- production mutation and order-state counters fixed at zero;
- `dashboard_order_controls_enabled=false`;
- `values_are_exchange_truth=false`.

## Classifications

```text
ok
missing_account_snapshot
portfolio_unavailable
shadow_intent_without_portfolio
production_mutation_forbidden
manual_review_required
```

## Allowed Actions

```text
record_only
mark_degraded
halt_shadow_flow
manual_review_required
```

These are local evidence actions only. They do not authorize exchange mutation.

## Forbidden Actions

The v0.12 reconciliation engine must never emit:

```text
submit_correction_order
cancel_production_order
replace_production_order
amend_production_order
retry_production_order
auto_flatten_position
```

## Verification

```bash
cargo test -p nautilus-cli production_readonly_reconciliation --lib
scripts/ai/verify_v12_production_readonly_reconciliation.sh
cargo clippy -p nautilus-cli --lib -- -D warnings
git diff --check
```
