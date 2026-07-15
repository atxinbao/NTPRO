# PTC-006 Historical Release Executable Retirement

Date: 2026-07-16
Executor: Codex

## Decision

The v0.32.0 backend baseline is frozen. Historical version-specific shell gates
are no longer current executable authority and are removed from `main` after
review. This is repository governance cleanup, not a backend patch release.

## Recovery Contract

`docs/rust-cutover/governance/historical_release_executable_retirement.json`
records every removed path with its pre-retirement source commit, Git blob SHA,
SHA-256, byte size, last change commit, classification, and exact restore
command. The Rust retirement guard reconstructs all blobs from Git and verifies
their identities on every current smoke run.

Immutable tags, published GitHub Releases, hosted workflow run references,
historical `docs/rust-cutover/` records, `tests/golden/` fixtures, and Rust
integration tests remain the historical audit authority.

## Current Authority

The retained source-tree authority is intentionally current-only:

- workspace Rust check, clippy, tests, docs, and product binary build;
- golden trace validation and Rust replay suites;
- Rust-only runtime and retired-source gates;
- docs/examples and control-plane governance;
- v0.32.0 backend freeze and current release-surface guards;
- release publication and publish-after-gate verification.

Requests for a later backend release require separately scoped governance. A
historical stage name passed to `verify_release.sh` fails instead of silently
replaying an obsolete contract.

## Boundaries

No v0.32.0 frozen release file is changed. No runtime, API, trading semantic,
adapter, mutation, retry, remediation, or live-exchange capability is added.
Python packaging and dependency cleanup remains blocked on PTC-006 review and
belongs to PTC-007.
