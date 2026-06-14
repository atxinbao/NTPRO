# NTPRO Versioning

Date: 2026-06-14
Executor: Codex

NTPRO uses several version-like identifiers. They do not mean the same thing.

## Plain Chinese Summary

大白话说：判断 NTPRO 当前发布能力时，看 `ntpro-rust-only-v*` release tag 和
release notes，不要只看 Cargo workspace version 或 `version.json` 徽章值。

`v0.6.0` 是当前正式公开发布点；`v0.6.1` 是当前 hardening 队列，不增加真实
Binance testnet 联网能力；`v0.7.0` 才计划进入真实 Binance testnet 只读连通性证明。

## Release Tags

Release tags are the product milestone identity.

Examples:

```text
ntpro-rust-only-v0.6.0
ntpro-rust-only-v0.6.1
ntpro-rust-only-v0.7.0
```

Use release tags and release notes to answer product questions such as:

- what NTPRO claims to support;
- what NTPRO explicitly does not support;
- which release gate evidence applies;
- whether a capability is public, absorbed, deferred, manual, or out of scope.

The current published release line is:

```text
ntpro-rust-only-v0.6.0
```

The active hardening track is:

```text
v0.6.1
```

v0.6.1 remains offline-only. It is for contract, Dashboard, artifact, and CI
hardening. It must not be described as real Binance testnet connectivity.

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

Planned boundary:

```text
real Binance testnet read-only connectivity proof
```

v0.7.0 must remain read-only:

- no order submission;
- no real account trading;
- no production trading claim;
- default CI stays offline;
- manual online gate is opt-in only;
- secrets must never be written to artifacts, stdout, logs, docs, or PR bodies.

## Rule Of Thumb

When these identifiers disagree, product-facing documentation must follow the
release tag and release notes. Cargo version and badge metadata are supporting
metadata only.
