# GitHub Automation Overview

Date: 2026-07-17
Executor: Codex

NTPRO repository automation is Rust-only. Current workflows do not install or
execute Python tooling and do not build or publish wheels, sdists, or Python
packages.

## Current Workflows

- `rust-cutover-smoke.yml`: required pull-request smoke, governance, CLI, and
  supervisor checks.
- `security-audit.yml`: Zizmor plus Cargo audit, deny, vet, and OSV scanning.
- `build.yml`: manually dispatched/current test-branch Rust formatting,
  workspace check, clippy, and split Rust tests.
- `cli-binaries.yml`: cross-platform Rust CLI binaries and optional R2 upload.
- `release-tag.yml` and `release-publish.yml`: frozen v0.32.0 backend release
  governance.
- `nightly-tests.yml`, `nightly-miri.yml`, `nightly-docs-features-check.yml`,
  and `dst.yml`: scheduled Rust diagnostics.
- `build-docs.yml`: retained upstream documentation dispatch; inactive in the
  standalone NTPRO repository.

## Composite Actions

- `common-setup`: Rust toolchain, Cargo cache, cargo-nextest, and optional prek
  setup.
- `cargo-tool-install`: pinned Cargo CLI installation and cache.
- `common-test-data`: shared Rust test data setup.
- `attest-sbom-retry`: release artifact attestation helper.

All third-party actions are pinned to immutable commit SHAs. Historical
workflow and package-publication records under `docs/rust-cutover/` remain
audit evidence and are not current automation authority.
