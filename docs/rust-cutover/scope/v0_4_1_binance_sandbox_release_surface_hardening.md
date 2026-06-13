# NTPRO v0.4.1 Scope - Binance Sandbox Release Surface Hardening

Date: 2026-06-13
Executor: Codex
Status: patch release contract

## Decision

`v0.4.1` is scoped as a patch-only release-surface hardening version for the
existing Binance Sandbox Product Foundation.

Release claim for `v0.4.1`:

```text
Binance Sandbox Product Foundation release-surface hardening
```

Plain language:

```text
v0.4.1 fixes and hardens how the v0.4 Binance sandbox release is presented,
verified, and published. It does not add new trading capability, real Binance
connectivity, real funds, real orders, or v0.5 workflow scope.
```

## Patch Boundary

In scope:

- README and public release wording aligned to the current patch tag.
- A clearly named v0.4 Binance sandbox smoke gate that reuses existing V04
  evidence tests.
- Hosted release-gate evidence for the v0.4.1 publish commit.
- v0.4.1 readiness report and release notes.
- Formal `ntpro-rust-only-v0.4.1` tag and GitHub Release after evidence agrees.

Out of scope:

- new Binance production trading claims;
- real funds;
- real account connectivity;
- real order submission;
- production Binance Spot or USDT-M parity;
- new runtime behavior;
- new adapter behavior;
- new Dashboard product scope;
- v0.5 planning or implementation work;
- prebuilt binary or Docker delivery as a v0.4.1 requirement.

## Relationship To v0.4.0

`v0.4.0` remains the first Binance Sandbox Product Foundation baseline.
`v0.4.1` does not supersede the capability boundary. It only makes the release
surface easier to audit:

```text
same Binance sandbox claim
same fixture/testnet/mock-only boundary
same no-real-funds/no-production-trading guarantee
clearer README + named smoke + hosted evidence + patch readiness notes
```

## Required Evidence

`v0.4.1` must not be tagged until all of these agree on the same publish
commit:

- README current source tag and capability wording;
- `scripts/ai/verify_v04_binance_sandbox.sh` result;
- hosted Rust Cutover Release Gate PASS;
- v0.4.1 readiness report;
- GitHub Release body.

## Release Surface Wording

README and release documents should describe `v0.4.1` with this wording:

```text
v0.4.1 is a patch-only release-surface hardening release for the Binance
Sandbox Product Foundation. It does not expand the v0.4.0 capability boundary.
It remains Binance sandbox-only: fixture/testnet/mock first, no real funds, no
production trading, and no real order submission.
```

Short label:

```text
Binance sandbox-only; release-surface hardening; no new trading capability.
```

## Task Sequence

```text
V041-001 v0.4.1 scope and release contract
  -> V041-002 publish README and tag release surface
  -> V041-003 explicit v0.4 Binance sandbox smoke gate
  -> V041-004 hosted release gate evidence
  -> V041-005 v0.4.1 readiness and release notes
  -> V041-006 formal v0.4.1 tag and GitHub Release
```

## Release Decision

Do not publish `ntpro-rust-only-v0.4.1` until `V041-001` through `V041-005`
have evidence and the final readiness report records strict PASS.
