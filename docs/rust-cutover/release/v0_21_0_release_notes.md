# NTPRO Rust-only v0.21.0 Release Notes

Date: 2026-07-01
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.21.0`
Release name: `NTPRO Rust-only v0.21.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.21.0`
Base release: `ntpro-rust-only-v0.20.1`
Published at: `2026-07-01T11:06:06Z`
Tag commit: `7e1cb46d692974bb5ef1398967c0927dd51c8091`
Hosted release gate: `https://github.com/atxinbao/NTPRO/actions/runs/28513012766`

## Summary

v0.21.0 is the Unified Read Model Foundation release. It establishes the
account, position, order, fill, risk, and Trader Terminal read-only Dashboard
foundation evidence chain for the Rust-only product surface.

This release does not expand production submit capability. It is a read-only
foundation release for unified read-model evidence, schema gates, golden
traces, release gates, and strict provenance.

## Publication Closeout

The formal GitHub Release `ntpro-rust-only-v0.21.0` is published and the hosted
release gate run `28513012766` completed successfully. The v0.21.0 issue set
`#651-#659` is closed, and milestone `#8` is closed with a live milestone
description that records the release URL, tag commit, hosted run, and boundary
claim.

The follow-up `v0.21.1` hardening patch (`#677-#682`) is the required blocker
before `v0.22.0` Trader Terminal workbench work (`#683-#690`) can start.

## Included Foundation Work

- `V210-000` / #651: v0.21 scope decision and v0.20.1 dependency gate.
- `V210-001` / #652: unified read model contract and schema.
- `V210-002` / #653: account snapshot read model.
- `V210-003` / #654: position read model and risk projection inputs.
- `V210-004` / #655: order lifecycle read model from submit/readback/cancel/audit evidence.
- `V210-005` / #656: fill and execution read model with dedupe and reconciliation.
- `V210-006` / #657: unified risk state projection.
- `V210-007` / #658: Trader Terminal read-only foundation Dashboard.
- `V210-008` / #659: v0.21 golden traces release gates and strict provenance.

## Release Gates

```text
scripts/ai/verify_release.sh v21-read-model-contract
scripts/ai/verify_release.sh v21-account-snapshot-read-model
scripts/ai/verify_release.sh v21-position-read-model
scripts/ai/verify_release.sh v21-order-lifecycle-read-model
scripts/ai/verify_release.sh v21-fill-execution-read-model
scripts/ai/verify_release.sh v21-risk-state-projection
scripts/ai/verify_release.sh v21-trader-terminal-readonly-dashboard
scripts/ai/verify_release.sh v21-release-gates
scripts/ai/verify_release.sh v21-strict-provenance
scripts/ai/verify_release_strict.sh v21
scripts/ai/check_github_release_published.sh
scripts/ai/check_release_surface_current.sh
```

## Boundary

v0.21.0 explicitly does not include:

- new production submit capability;
- production order mutation;
- implicit retry;
- automatic cancel;
- automatic remediation;
- automatic risk action or repair;
- automatic position repair or auto-flatten;
- execution algorithm;
- retry, replace, amend, correction, or flatten;
- strategy-driven production execution;
- multi-account or multi-venue execution expansion;
- real-funds proof in CI;
- product-grade live trading terminal readiness;
- executable read-model runtime readiness;
- Trader Terminal workbench readiness;
- Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or order-ticket controls.

## Scope Claim

v0.21.0 provides a unified read model foundation and a read-only Trader
Terminal Dashboard foundation. The release scope is `unified_read_model_foundation`.
The Dashboard display claim is `read_only_foundation`, not product-grade live
trading terminal readiness.
