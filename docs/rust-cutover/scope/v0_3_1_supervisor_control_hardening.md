# NTPRO v0.3.1 Scope Addendum - Local Supervisor Control Console Hardening

Date: 2026-06-12
Executor: Codex
Status: proposed patch hardening milestone

## Decision

`ntpro-rust-only-v0.3.0` already established the local sandbox-only
`Local Supervisor Control Console` capability. The next patch release should not
expand the product claim again. It should harden the released surface so the
published README, release gate, hosted verification evidence, and local control
semantics all say the same thing.

Release claim for `v0.3.1`:

```text
Local Supervisor Control Console Hardening
```

Plain language:

```text
v0.3.1 is not a new trading product milestone.
It is the patch release that cleans up the public release surface, tightens
release verification, and makes the local control-console semantics harder to
misread.
```

## In Scope

- Update the public README and release-facing wording so the current published
  milestone no longer advertises `v0.2.0`.
- Make v0.3 supervisor control smoke and dashboard smoke runnable against
  `target/release` binaries from `scripts/ai/verify_release.sh`.
- Record hosted GitHub Actions evidence for the release gate and close the gap
  between local PASS evidence and remote run visibility.
- Harden local supervisor process bookkeeping:
  - stale registry lock recovery;
  - stronger process identity evidence beyond bare PID.
- Clarify local control semantics:
  - pause/resume are artifact-level local supervisor controls;
  - reconnect actions remain explicit local sandbox `not_supported` results.
- Add negative-path API and CLI tests for unsupported or invalid control
  requests.
- Close the first batch of high-risk ignored tests or replace them with
  deterministic coverage.
- Produce a strict PASS/FAIL v0.3.1 readiness report.

## Out Of Scope

- production real-exchange live trading;
- real account connectivity;
- real order submission;
- manual order entry;
- production reconnect behavior;
- runtime-level pause or resume for live strategy, adapter, or execution loops;
- multi-user permissions;
- remote or distributed dashboard operation;
- Docker or prebuilt binary delivery as a v0.3.1 requirement;
- v0.4 exchange or strategy productization.

## Current Problems To Close

- The published `ntpro-rust-only-v0.3.0` README still advertises
  `ntpro-rust-only-v0.2.0`.
- The release gate invokes v0.3 smoke with skip-build enabled while the smoke
  scripts still read `target/debug` binaries.
- Hosted release-gate evidence is not closed out in the release narrative.
- Pause/resume can be misread as runtime suspension instead of local artifact
  control.
- Reconnect controls can be misread as real venue reconnect instead of explicit
  local sandbox `not_supported`.

## Required v0.3.1 Outcomes

Before publishing `ntpro-rust-only-v0.3.1`, the patch must prove:

```text
README and release-facing docs match the current source tag and capability.
verify_release can drive v0.3 smoke against release binaries.
Hosted release-gate evidence is recorded and understandable.
Pause/resume and reconnect semantics are documented and test-backed.
Local invalid control requests return explicit negative-path results.
The first ignored-test hardening batch is either closed or explicitly replaced.
```

## Task Sequence

```text
V031-001 Public README and release surface cleanup
  -> V031-002 Release binary smoke support
  -> V031-003 Hosted release gate triage and evidence closeout
  -> V031-004 Registry stale lock recovery
  -> V031-005 Process identity hardening
  -> V031-006 Pause/resume semantics contract
  -> V031-007 Reconnect control contract cleanup
  -> V031-008 Negative control API tests
  -> V031-009 Ignored tests closure batch 1
  -> V031-010 v0.3.1 readiness report
```

## Release Decision

Do not publish `ntpro-rust-only-v0.3.1` until `V031-001` through `V031-010`
have evidence and the final readiness report records strict PASS.
