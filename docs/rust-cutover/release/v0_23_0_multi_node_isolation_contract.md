# v0.23.0 Multi-Node Isolation Contract

Date: 2026-07-03
Executor: Codex
Task: `V230-001`
GitHub issue: `#712`
Milestone: `v0.23.0`
Status: CONTRACT_DEFINED

## Summary

`v0.23.0` may begin scoped multi-account, multi-strategy, and multi-venue node
isolation work only through the contract below. The contract defines stable
identity keys, boundary rules, allowed read paths, future owner-approved control
paths, evidence requirements, and release claim limits before any runtime
implementation starts.

Plain Chinese summary: v0.23.0 的核心不是先做更多交易按钮，而是先保证账户、策略、
venue node、权限、日志、证据和 Dashboard 视图不会串线。本合同要求每条下游实现都带
清楚的 `account_key`、`strategy_key`、`venue_node_key` 和 `isolation_scope_key`，
缺失或冲突时必须 fail closed 或降级为不可用；真实操作仍然不能绕过 owner approval、
risk gate 和 audit gate。

## Entry Gate

```text
v0.22.1 release tag = ntpro-rust-only-v0.22.1
v0.22.1 release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.22.1
v0.22.1 release published at = 2026-07-03T09:40:04Z
v0.22.1 hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28647486521
v0.22.1 hosted release gate conclusion = success
V230-000 issue = #711 closed
V230-001 issue = #712 contract definition
```

This entry gate authorizes only scoped V230 implementation work. It does not
authorize production submit, production order mutation, strategy-driven live
execution, or Dashboard operation controls.

## Identity Model

Every downstream V230 artifact that represents runtime state, read-model state,
operator-visible state, logs, or evidence must carry explicit stable identity
keys.

| Identity | Stable key | Required source | Boundary rule |
| --- | --- | --- | --- |
| Account | `account_key` | Owner-configured account identity plus venue/environment disambiguation | Never infer from display name, default account, credential alias, or venue alone. |
| Strategy | `strategy_key` | Owner-configured strategy identity plus deployment/session disambiguation | Never infer from account, instrument, venue, or process name alone. |
| Venue node | `venue_node_key` | Owner-configured venue, environment, node identity, and adapter instance identity | Never infer from adapter type, exchange name, or process id alone. |
| Isolation scope | `isolation_scope_key` | Composite of `account_key`, `strategy_key`, and `venue_node_key` | Required for any cross-node view, approval artifact, audit row, or evidence package. |

Key requirements:

```text
account_key is required for account, position, order, fill, risk, and account-scoped evidence.
strategy_key is required for strategy supervisor state, strategy logs, and strategy-scoped evidence.
venue_node_key is required for venue node lifecycle, adapter state, connection state, and venue-scoped evidence.
isolation_scope_key is required when data crosses account, strategy, venue, node, dashboard, log, or evidence boundaries.
missing_key_behavior = fail_closed_or_degraded_unavailable
mismatched_key_behavior = fail_closed
default_account_fallback = forbidden
default_strategy_fallback = forbidden
default_venue_node_fallback = forbidden
```

## Isolation Boundaries

### Account Boundary

Account-scoped state must remain partitioned by `account_key`.

```text
positions must not merge across account_key
orders must not merge across account_key
fills must not merge across account_key
risk state must not merge across account_key
approval artifacts must not be consumed across account_key
account logs must preserve account_key
account evidence must preserve account_key
```

### Strategy Boundary

Strategy-scoped state must remain partitioned by `strategy_key`.

```text
strategy supervisor state must not merge across strategy_key
strategy decisions must not borrow another strategy_key
strategy audit rows must preserve strategy_key
strategy evidence must preserve strategy_key
shared strategy approval consumption = forbidden
strategy-driven production execution = forbidden until a later scoped release explicitly gates it
```

### Venue Node Boundary

Venue node state must remain partitioned by `venue_node_key`.

```text
node lifecycle state must preserve venue_node_key
adapter status must preserve venue_node_key
connection state must preserve venue_node_key
venue node logs must preserve venue_node_key
venue node evidence must preserve venue_node_key
cross-venue implicit operation = forbidden
cross-node implicit operation = forbidden
```

### Adapter Boundary

Adapter evidence may be read and aggregated only when every row keeps its
identity and provenance.

```text
adapter payload source provenance = required
adapter fixture provenance = required for behavior changes
adapter behavior change without fixture evidence = forbidden
adapter state without venue_node_key = invalid
adapter state without account_key when account-scoped = invalid
```

### Dashboard Boundary

Dashboard work in v0.23.0 may aggregate read-only views only when each row keeps
its identity labels and provenance. It must not introduce mutation controls.

```text
dashboard read-only aggregation = allowed with per-row identity labels
dashboard filter by account_key = required for account-scoped data
dashboard filter by strategy_key = required for strategy-scoped data
dashboard filter by venue_node_key = required for venue-scoped data
dashboard operation controls = forbidden
dashboard submit/cancel/retry/replace/amend/flatten/order-ticket controls = forbidden
dashboard missing identity behavior = degraded_unavailable
```

### Logs And Evidence Boundary

Logs and evidence must be traceable to the same identity model used by runtime
and read-model artifacts.

```text
audit rows must include isolation_scope_key or explicit scoped keys
provenance rows must include source and scoped keys
evidence artifacts must name covered account_key/strategy_key/venue_node_key or state not_applicable
release evidence must list downstream issue traceability
ambiguous evidence scope = invalid
```

## Allowed Read Paths

The following read paths are allowed for V230 implementation issues when they
preserve the identity model and provenance:

```text
single_account_read = allowed with account_key
single_strategy_read = allowed with strategy_key
single_venue_node_read = allowed with venue_node_key
cross_node_read_model_aggregation = allowed with isolation_scope_key per row
operator_dashboard_read = allowed with visible account/strategy/venue node labels
release_evidence_read = allowed with issue and identity traceability
```

Read path failure rules:

```text
missing account_key on account-scoped data = fail_closed_or_degraded_unavailable
missing strategy_key on strategy-scoped data = fail_closed_or_degraded_unavailable
missing venue_node_key on venue-scoped data = fail_closed_or_degraded_unavailable
cross-scope key mismatch = fail_closed
mixed-source row without provenance = fail_closed_or_degraded_unavailable
```

## Future Owner-Approved Control Paths

V230-001 does not implement control paths. It defines the minimum contract for
any later task that explicitly proposes a control path.

```text
owner_approval_gate = required per isolation_scope_key
risk_gate = required per isolation_scope_key
audit_gate = required per isolation_scope_key
approval_consumption = single_scope_only
shared_approval_consumption = forbidden
implicit_cross_account_operation = forbidden
implicit_cross_strategy_operation = forbidden
implicit_cross_venue_operation = forbidden
implicit_cross_node_operation = forbidden
automatic_cancel = forbidden
automatic_remediation = forbidden
ungated_submit_cancel_retry_replace_amend_flatten = forbidden
```

Any future control artifact must prove:

```text
owner identity
target account_key
target strategy_key when strategy-scoped
target venue_node_key
target isolation_scope_key
risk decision provenance
audit row provenance
single-use approval consumption
failure behavior on missing or mismatched keys
```

## Downstream Traceability

Every downstream V230 issue must cite this contract and record the specific
sections it satisfies.

| Issue | Task | Required contract sections |
| --- | --- | --- |
| #713 | V230-002 multi-account runtime identity and read-model partitioning | Identity Model, Account Boundary, Allowed Read Paths, Logs And Evidence Boundary |
| #714 | V230-003 multi-strategy supervisor identity and isolation | Identity Model, Strategy Boundary, Future Owner-Approved Control Paths, Logs And Evidence Boundary |
| #715 | V230-004 multi-venue node registry and lifecycle boundary | Identity Model, Venue Node Boundary, Adapter Boundary, Logs And Evidence Boundary |
| #716 | V230-005 multi-node orchestration and control-plane gating | Identity Model, Future Owner-Approved Control Paths, Isolation Boundaries, Logs And Evidence Boundary |
| #717 | V230-006 multi-account strategy venue dashboard and observability surface | Dashboard Boundary, Allowed Read Paths, Logs And Evidence Boundary |
| #718 | V230-007 v0.23.0 release gates and strict provenance | Entry Gate, Downstream Traceability, Validation Requirements, Release Claims |

No downstream issue may claim v0.23.0 release readiness until all preceding
issue-specific evidence has merged.

## Validation Requirements

Every downstream V230 implementation PR must provide evidence for the applicable
items below:

```text
identity_keys_present
isolation_scope_key_present_when_crossing_boundaries
missing_key_fail_closed_or_degraded_unavailable
mismatched_key_fail_closed
no_default_account_fallback
no_default_strategy_fallback
no_default_venue_node_fallback
read_path_preserves_provenance
approval_consumption_single_scope_only
dashboard_has_no_operation_controls
adapter_behavior_change_has_fixture_or_scope_note
release_claims_do_not_exceed_contract
```

Behavior-changing downstream PRs must include targeted Rust tests, golden trace
coverage, adapter fixture evidence, or a scoped evidence note explaining why a
local automated proof is not applicable.

## Release Claims

Allowed v0.23.0 claim if all downstream issues pass:

```text
v0.23.0 provides multi-account, multi-strategy, and multi-venue node isolation boundaries and read-only observability with explicit identity, provenance, and gate evidence.
```

Forbidden v0.23.0 claims:

```text
product-grade live trading terminal
unattended production trading platform
new production submit capability
production order mutation expansion
strategy-driven production execution
automatic cancel
automatic remediation
cross-account implicit operation
cross-strategy implicit operation
cross-venue implicit operation
shared approval consumption across isolated nodes
Dashboard operation controls
complete executable read-model runtime coverage unless separately proven
```

## Non-Goals

This contract does not implement runtime code, adapter behavior, Dashboard
views, release tags, GitHub Release publication, submit, cancel, retry, replace,
amend, flatten, automatic remediation, or strategy-driven live trading.
