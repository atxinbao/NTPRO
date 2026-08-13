# GitHub Automation Overview

Date: 2026-07-24
Executor: Codex

NTPRO repository automation is Rust-only. Current workflows do not install or
execute Python tooling and do not build or publish wheels, sdists, or Python
packages.

## Current Workflows

- `rust-cutover-smoke.yml`: required pull-request gate with parallel core,
  Rust lint, and Rust test lanes, aggregated by the stable `smoke` check.
- `security-audit.yml`: Zizmor plus Cargo audit, deny, vet, and OSV scanning.
- `backend-performance.yml`: scheduled or manually dispatched backend
  performance baseline and regression contract; ordinary pull requests do not
  run its six-workload matrix.
- `release-tag.yml`: tag-push release gate for the current Rust-only release
  surface.
- `release-publish.yml`: publishes a release only after a successful matching
  tag-push gate.

The v0.32.0 backend baseline remains frozen. The current release authority is
v0.33.0 Backend Maintenance. Retired upstream, nightly, optional binary, and
unreachable self-hosted workflows remain available only through Git history.

## Composite Actions

- `common-setup`: Rust toolchain, Cargo cache, cargo-nextest, and optional prek
  setup.
- `cargo-tool-install`: pinned Cargo CLI installation and cache.
- `common-test-data`: shared Rust test data setup.
- `attest-sbom-retry`: release artifact attestation helper.

All third-party actions are pinned to immutable commit SHAs. Historical
workflow and package-publication records under `docs/rust-cutover/` remain
audit evidence and are not current automation authority.
