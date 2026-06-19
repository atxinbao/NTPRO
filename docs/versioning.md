# NTPRO Versioning

Date: 2026-06-14
Executor: Codex

NTPRO uses several version-like identifiers. They do not mean the same thing.

## Plain Chinese Summary

大白话说：判断 NTPRO 当前发布能力时，看 `ntpro-rust-only-v*` release tag 和
release notes，不要只看 Cargo workspace version 或 `version.json` 徽章值。

`v0.10.0` 是当前正式公开发布点；`v0.10.1` 是 release-surface hotfix 队列，
不增加生产 Binance、真实资金、生产交易、自动在线下单或 Dashboard 下单按钮；
`v0.11.0` 才是下一条 Production Read-Only + Shadow Portfolio 能力线。

## Release Tags

Release tags are the product milestone identity.

Examples:

```text
ntpro-rust-only-v0.6.0
ntpro-rust-only-v0.6.1
ntpro-rust-only-v0.7.0
ntpro-rust-only-v0.10.0
```

Use release tags and release notes to answer product questions such as:

- what NTPRO claims to support;
- what NTPRO explicitly does not support;
- which release gate evidence applies;
- whether a capability is public, absorbed, deferred, manual, or out of scope.

The current published release line is:

```text
ntpro-rust-only-v0.10.0
```

The active patch track is:

```text
v0.10.1
```

v0.10.1 is a release-surface hotfix track. It must not be described as
production Binance connectivity, real funds, production trading, automatic
online order mutation, or Dashboard order controls.

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

Planned boundary:

```text
Production Read-Only + Shadow Portfolio
```

v0.11.0 must remain read-only/shadow-only:

- no production order submission;
- no production cancel, replace, amend, or automatic correction orders;
- no real-funds trading;
- no Dashboard order controls;
- default CI stays offline;
- manual online production read-only proof is opt-in only;
- secrets must never be written to artifacts, stdout, logs, docs, or PR bodies.

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
