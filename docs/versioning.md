# NTPRO Versioning

Date: 2026-06-27
Executor: Codex

NTPRO uses several version-like identifiers. They do not mean the same thing.

## Plain Chinese Summary

大白话说：判断 NTPRO 当前发布能力时，看 `ntpro-rust-only-v*` release tag 和
release notes，不要只看 Cargo workspace version 或 `version.json` 徽章值。

`v0.25.0` 是当前正式公开发布点；它是 Monitoring, Incident, and
Disaster-Recovery Foundation 发布线。它建立在 v0.24.1 基础上，收口 intake、
monitoring observability、alert taxonomy/routing、incident lifecycle、
runbook/audit、DR preview、read-only Dashboard monitoring、SLO/freshness、
release gates 和 strict provenance。
它不是产品级实盘交易终端，不是新增 submit 能力，不是生产订单 mutation，不调用
execution adapter、adapter send 或 live exchange request，不是隐式 retry，不启用
retry scheduler，不是自动补救或自动恢复，不是策略实盘，也没有 Dashboard
下单/审批/撤单/重试/submit/replace/amend/flatten/remediation/order-ticket 控件。
`v0.26.0` 是下一能力轨占位。

## Release Tags

Release tags are the product milestone identity.

Examples:

```text
ntpro-rust-only-v0.6.0
ntpro-rust-only-v0.6.1
ntpro-rust-only-v0.7.0
ntpro-rust-only-v0.10.0
ntpro-rust-only-v0.11.0
ntpro-rust-only-v0.12.0
ntpro-rust-only-v0.12.1
ntpro-rust-only-v0.13.0
ntpro-rust-only-v0.14.0
ntpro-rust-only-v0.15.0
ntpro-rust-only-v0.16.0
ntpro-rust-only-v0.17.0
ntpro-rust-only-v0.18.0
ntpro-rust-only-v0.19.0
ntpro-rust-only-v0.20.0
ntpro-rust-only-v0.20.1
ntpro-rust-only-v0.21.0
ntpro-rust-only-v0.21.1
ntpro-rust-only-v0.22.0
ntpro-rust-only-v0.22.1
ntpro-rust-only-v0.23.0
ntpro-rust-only-v0.23.1
ntpro-rust-only-v0.24.0
ntpro-rust-only-v0.24.1
ntpro-rust-only-v0.25.0
```

Use release tags and release notes to answer product questions such as:

- what NTPRO claims to support;
- what NTPRO explicitly does not support;
- which release gate evidence applies;
- whether a capability is public, absorbed, deferred, manual, or out of scope.

The current published release line is:

```text
ntpro-rust-only-v0.25.0
```

The active patch track is:

```text
v0.25.1
```

v0.25.1 is reserved for future patch hardening only. It must not expand beyond
the v0.25.0 monitoring / incident / DR foundation boundary unless a later
scoped release issue explicitly changes that contract.

The next capability track is:

```text
v0.26.0
```

v0.26.0 is a placeholder for the next capability track. It does not inherit
production submit, production order mutation, execution adapter send, adapter
send, live exchange request, implicit retry, retry scheduler, automatic
remediation/recovery, strategy-driven production execution, shared approval
consumption, or Dashboard operation controls from v0.25.0.

## Cargo Workspace Version

The Cargo workspace version is the Rust package metadata version in
`Cargo.toml`:

```toml
[workspace.package]
version = "0.58.0"
```

This value is inherited from the Rust crate lineage and dependency metadata. It
is useful for Cargo package compatibility, but it is not the NTPRO product
milestone and does not prove the current release capability.

Use `scripts/package-version.sh` when a script needs the Cargo workspace
package version.

## version.json

`version.json` is badge/display metadata:

```json
{
  "schemaVersion": 1,
  "message": "v1.228.0"
}
```

This value is not the NTPRO release line. It must not be used as evidence that
the NTPRO product supports a feature or matches a GitHub Release.

## Current Capability Boundaries

### v0.10.0

Includes:

- Binance spot sandbox order proof;
- owner-gated Spot Demo Mode submit/cancel evidence;
- redacted request, ack, lifecycle, reconciliation, and summary artifacts;
- terminal `CANCELED` reconciliation evidence;
- read-only Dashboard order-proof display;
- production order counters fixed at zero.

Does not include:

- production Binance connectivity;
- production order submission, cancel, replace, amend, or live order
  management;
- automatic online order mutation;
- real funds;
- production trading;
- Dashboard order controls.

### v0.11.0

Published boundary:

```text
Production Read-Only Contract + Offline Shadow Portfolio
```

Includes:

- production endpoint classification;
- production public read-only contract, offline fail-closed;
- owner-gated authenticated account snapshot contract, offline fail-closed;
- local shadow execution intent artifacts;
- local shadow portfolio snapshot artifacts;
- local shadow/read-only lifecycle state evidence;
- local reconciliation/manual-remediation event evidence;
- read-only Dashboard production shadow status;
- offline release gates.

Does not include:

- no production order submission;
- no production cancel, replace, amend, or automatic correction orders;
- no production open-order or order-state reads such as `/api/v3/openOrders`;
- no successful online production public/account reads;
- no production network-read runtime as completed capability;
- no real-funds trading;
- no Dashboard order controls;
- default CI stays offline;
- any future manual online production read-only proof requires a separate
  owner-gated task and is not completed by v0.11.0;
- secrets must never be written to artifacts, stdout, logs, docs, or PR bodies.

### v0.12.0

Published boundary:

```text
Production Online Read-Only + Persistent Shadow
```

Includes:

- owner-gated production public read-only online proof;
- owner-gated authenticated production account snapshot read-only proof;
- redacted production account response-shape evidence;
- local shadow portfolio runtime artifact;
- local persistent shadow strategy session event artifact;
- local read-only reconciliation classifications;
- Dashboard v0.12 production shadow read-only panel;
- v0.12 offline release gates and manual-online fail-closed preflight.

Does not include:

- no production order submission;
- no production cancel, replace, amend, retry, or automatic correction orders;
- no production open-order or order-state reads;
- no listenKey lifecycle access;
- no strategy-driven production execution;
- no automatic production remediation;
- no production portfolio parity;
- no real-funds trading;
- no Dashboard order controls;
- default CI stays offline;
- owner-gated online production read-only proof is optional and not required for
  CI.

### v0.13.0

Current boundary:

```text
Guarded Live Alpha Preflight only
```

V130-001 is the owner-approved scope decision for the v0.13.0 line. It defines
v0.13.0 as preflight evidence only, not production order submission, production
order mutation, production order-state reads, listenKey lifecycle, real-funds
trading, production trading, or Dashboard order controls. It must not be
described as risk/execution-grade live-alpha money math readiness.

### v0.14.0

Current boundary:

```text
Production Order-State Read-Only + Live Alpha Dry-Run only
```

V140-000 defines the owner-approved v0.14.0 boundary. It allows owner-gated
production order-state read-only proof scope and local live-alpha dry-run
evidence only. It is not production order submission, production order
mutation, cancel/replace/amend/retry/correction, listenKey lifecycle,
real-funds trading, production trading, automatic remediation, or Dashboard
order controls.

### v0.15.0

Current boundary:

```text
Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness only
```

V150-000 defines the owner-approved v0.15.0 boundary. It allows production
mutation endpoint classification, redacted local request-preview artifacts,
manual approval lifecycle, kill-switch runtime gating, local dry-run execution
adapter evidence, incident/rollback evidence, and Dashboard read-only mutation
preflight only. It is not production request sending, production order
submission, production order mutation, cancel/replace/amend/retry/correction,
listenKey lifecycle, real-funds trading, production trading, automatic
remediation, or Dashboard order controls.

### v0.17.0

Published boundary:

```text
Production Reconciliation And Orphan Recovery Evidence
```

V170-000 defines the v0.17.0 scope, and the formal release keeps that boundary.
v0.17.0 includes local/offline evidence for the v0.16 single candidate lineage:
local ledger persistence, redacted exchange readback mapping, reconciliation
classification, orphan order risk detection, restart recovery, read-only
Dashboard evidence, and failure incident semantics. It must not be treated as
network readback execution, new production order submission, production order
mutation, actual cancel send, automatic orphan cleanup, strategy-driven
production execution, multi-account/multi-venue execution, real-funds trading
platform readiness, or Dashboard order/cancel controls.

### v0.18.0

Current boundary:

```text
Owner-Approved Cancel Recovery Preview
```

V180-000 defines the v0.18.0 scope, and the formal release keeps that boundary.
v0.18.0 includes preview-only cancel recovery evidence for the v0.17
reconciliation line: cancel intent contracts, owner approval lifecycle,
preview request/response evidence, post-cancel readback contracts, failure and
partial-success semantics, rollback evidence, release gates, and read-only
Dashboard diagnostics. It must not be treated as actual cancel send, automatic
cancel, retry/replace/amend/correction/flatten/remediation, strategy-driven
production execution, multi-account/multi-venue execution, real-funds trading
platform readiness, or Dashboard order/cancel controls.

### v0.20.0

Published boundary:

```text
Owner-Approved Production Order Lifecycle Foundation
```

V200-000 through V200-012 define and release the v0.20.0 scope. v0.20.0
includes only the owner-approved production order lifecycle foundation with
pre-submit risk evidence, owner approval, env-only signing-material readiness,
one guarded submit attempt, response redaction, post-submit readback
reconciliation, failure/no-retry evidence, read-only Dashboard order lifecycle
audit, golden trace coverage, aggregate release gates, and strict provenance.
It must not be treated as product-grade live trading, implicit retry, automatic
cancel, automatic remediation, bulk order execution, retry/replace/amend/
correction/flatten, strategy-driven production execution, multi-account/
multi-venue execution, real-funds trading platform readiness, or Dashboard
operation controls.

### v0.20.1

Published boundary:

```text
Production Order Lifecycle Release Closeout & Provenance Hardening Patch
```

V201-001 through V201-007 define and release the v0.20.1 hardening patch. It
backfills v0.20.0 publication evidence, hardens V20 provenance, adds durable
submit-attempt ledger proof, recomputes pre-submit notional consistency, labels
adapter/readback source provenance, displays Dashboard foundation-boundary
diagnostics, and records v0.21.0 dependency proof. It must not be treated as new
submit capability, product-grade live trading, implicit retry, automatic cancel,
automatic remediation, bulk order execution, retry/replace/amend/correction/
flatten, strategy-driven production execution, multi-account/multi-venue
execution, real-funds trading platform readiness, or Dashboard operation
controls.

### v0.19.0

Published boundary:

```text
Owner-Approved Single-Shot Actual Cancel
```

V190-001 through V190-010 define and release the v0.19.0 scope. v0.19.0
includes one owner-approved actual cancel path with one approval, one order,
one venue, one execution attempt, risk gate evidence, adapter boundary
evidence, post-cancel readback reconciliation, failure and partial-success
evidence, read-only Dashboard actual cancel audit, golden trace coverage, and
release gates. It must not be treated as production order submit lifecycle,
automatic cancel, bulk cancel, retry/replace/amend/correction/flatten/
remediation, second cancel, compensation trade, strategy-driven production
execution, multi-account/multi-venue execution, real-funds trading platform
readiness, or Dashboard order/cancel controls.

### v0.16.0

Current boundary:

```text
Minimum Owner-Approved Production Order Mutation Candidate
```

V160-001 defines the v0.16.0 scope, and the formal release keeps that boundary.
v0.16.0 includes only one owner-approved tiny `LIMIT` `GTC` production order
candidate with default fail-closed gates, owner-gated signing material,
redacted request/response artifacts, guarded send, kill-switch enforcement,
post-submit readback, audit trail, terminal no-retry failure semantics, and no
Dashboard order controls. It must not inherit strategy-driven production
execution, multiple orders, `MARKET` orders, cancel/replace/amend/retry/
correction/flatten, listenKey lifecycle, multi-venue/multi-account execution,
real-funds trading platform claims, automatic remediation, or Dashboard order
controls from earlier request-preview/dry-run evidence.

### v0.6.0

Includes:

- offline Binance testnet dry-run runtime foundation;
- local workflow artifacts;
- Dashboard read-only artifact surfaces;
- no real funds;
- no real order submission;
- no production trading.

### v0.6.1

Includes:

- release wording alignment;
- Dashboard copy cleanup;
- workflow run ID contract hardening;
- offline-only connectivity-probe wording;
- artifact browser and manifest health hardening;
- PR-stage smoke coverage;
- readiness report and release notes.

Does not include:

- real Binance testnet network connection;
- real Binance testnet order submission;
- production trading;
- Dashboard network controls;
- credential-reading Dashboard behavior.

### v0.7.0

Published boundary:

```text
real Binance testnet read-only connectivity proof
```

v0.7.0 remains read-only:

- no order submission;
- no real account trading;
- no production trading claim;
- default CI stays offline;
- manual online gate is opt-in only;
- secrets must never be written to artifacts, stdout, logs, docs, or PR bodies.

### v0.8.0

Includes:

- authenticated Binance testnet read-only account-shape proof;
- env-var-only credential policy;
- redacted account-shape evidence;
- no order submission;
- no account mutation;
- no production trading.

### v0.9.0

Includes:

- local deterministic Strategy Runtime batch foundation;
- fixture/mock market input;
- signal artifacts;
- shadow order-intent and risk-decision artifacts;
- supervisor and Dashboard read-only status/artifact display.

Does not include:

- persistent long-running Strategy Runtime semantics;
- Binance sandbox order submission;
- production order submission;
- real funds;
- production trading;
- Dashboard order controls.

## Rule Of Thumb

When these identifiers disagree, product-facing documentation must follow the
release tag and release notes. Cargo version and badge metadata are supporting
metadata only.
