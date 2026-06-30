# NTPRO Rust-only v0.20.1 Release Notes

Date: 2026-06-30
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.20.1`
Release name: `NTPRO Rust-only v0.20.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.20.1`
Base release: `ntpro-rust-only-v0.20.0`

## Summary

v0.20.1 is the Production Order Lifecycle Release Closeout & Provenance Hardening Patch.
It backfills published v0.20.0 release evidence, hardens V20 provenance, adds a
durable submit-attempt ledger, recomputes pre-submit notional consistency,
labels adapter/readback evidence source provenance, and adds Dashboard
foundation-boundary diagnostics.

This patch does not expand production submit capability. It remains a
foundation-only release for owner-approved production order lifecycle evidence.

## Included Hardening

- `V201-001` / #644: v0.20.0 release closeout and publication evidence backfill.
- `V201-002` / #645: V20 provenance hardening across tests, fixtures, and golden traces.
- `V201-003` / #650: durable single-shot attempt ledger and atomic approval consumption.
- `V201-004` / #646: pre-submit notional consistency hardening.
- `V201-005` / #647: adapter source and readback provenance labeling.
- `V201-006` / #648: Dashboard diagnostics hardening for foundation boundaries.
- `V201-007` / #649: v0.20.1 release gates and dependency proof.

## Release Gates

```text
scripts/ai/verify_release.sh v20-release-gates
scripts/ai/verify_release.sh v20-strict-provenance
scripts/ai/verify_release.sh v20.1-release-gates
scripts/ai/verify_v20_patch_release_gates.sh
scripts/ai/check_github_release_published.sh
scripts/ai/check_release_surface_current.sh
```

## Boundary

v0.20.1 explicitly does not include:

- new production submit capability;
- implicit retry;
- automatic cancel;
- automatic remediation;
- bulk order execution;
- retry, replace, amend, correction, or flatten;
- strategy-driven production execution;
- multi-account or multi-venue execution;
- real-funds proof in CI;
- product-grade live trading terminal readiness;
- Dashboard order, approval, cancel, or retry controls.

## v0.21.0 Dependency

The v0.21.0 start gate was satisfied on 2026-06-30 after all V201 issues were
closed and this v0.20.1 release evidence was published. V210 work may start
from its own scoped issue order; it must not inherit any submit expansion from
v0.20.1.
