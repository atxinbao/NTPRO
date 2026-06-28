# NTPRO Rust-only v0.19.0 Release Notes

Date: 2026-06-28
Executor: Codex
Status: RELEASE CANDIDATE - pending V190-010 PR merge and tag publication
Planned tag: `ntpro-rust-only-v0.19.0`
Release name: `NTPRO Rust-only v0.19.0`

## Summary

`v0.19.0` is the owner-approved single-shot actual cancel release candidate.
It promotes the v0.18 preview evidence into a tightly gated manual execution
line: the cancel is allowed only when owner approval, risk gate, adapter
boundary, release provenance, one-order identity, one-venue scope, readback,
failure evidence, Dashboard read-only audit, and golden trace coverage all
agree.

Plain Chinese summary: v0.19.0 是“Owner 人工批准的一次性真实撤单”候选版本。大白话：
这版只允许 owner-approved single-shot actual cancel：一个审批、一个订单、一个 venue、
一次撤单尝试，并且必须有 risk gate、adapter boundary、readback、failure evidence 和
golden trace 证明。它不是生产下单生命周期版本，不允许自动撤单、批量撤单、Dashboard 撤单按钮、
retry、二次撤单或自动补救。

## Product Claim

```text
capability = Owner-Approved Single-Shot Actual Cancel
actual cancel only owner-approved single-shot
owner approval = required
approval reuse = forbidden
risk gate = required
adapter boundary = required
single order = required
single venue = required
single execution attempt = required
post-cancel readback = required
failure evidence = required
Dashboard surface = read-only audit view
golden traces = included
aggregate release gate = v19-release-gates
production order submit lifecycle = not included
automatic cancel = not included
bulk cancel = not included
Dashboard cancel button = not included
```

## Changed

- Added the v0.19 actual cancel safety contract.
- Added owner approval lifecycle evidence for single-use manual approval.
- Added the single-shot actual cancel command boundary.
- Added cancel executor adapter boundary and capability evidence.
- Added post-cancel readback reconciliation evidence.
- Added failure, partial-success, timeout, unknown, rejected, recovered, and
  degraded outcome evidence.
- Added Dashboard actual cancel audit as a read-only view.
- Added actual cancel golden traces and Rust harness coverage.
- Added `v19-release-gates` for PR and release-tag verification.

## Release-Blocking Conditions

The v0.19 release guard rejects the package if any of these conditions appear:

```text
missing owner approval = release-blocking
missing approval provenance = release-blocking
reused approval = release-blocking
scope mismatch = release-blocking
missing risk gate = release-blocking
missing adapter boundary = release-blocking
missing readback = release-blocking
missing failure evidence = release-blocking
automatic cancel = release-blocking
bulk cancel = release-blocking
Dashboard cancel button = release-blocking
retry / second cancel / remediation = release-blocking
raw secret or raw response persistence = release-blocking
production order submit lifecycle = release-blocking
```

## Not Included

```text
production order submit lifecycle = not included
v0.20 enters owner-approved production order lifecycle
automatic cancel = not included
automatic remediation = not included
bulk cancel = not included
cancel all open orders = not included
multi-account cancel = not included
multi-strategy cancel = not included
multi-venue cancel = not included
retry / replace / amend / flatten = not included
second cancel = not included
compensation trade = not included
Dashboard cancel button = not included
Dashboard approval button = not included
Dashboard credential input = not included
general production trading platform claim = not included
```

## Validation

Required release validation:

```text
scripts/ai/verify_release.sh v19-release-gates
scripts/ai/verify_v19_release_gates.sh
scripts/ai/verify_v19_actual_cancel_golden_traces.sh
scripts/ai/run_golden_traces.sh
python3 scripts/ai/validate_golden_trace_release_scope.py --manifest docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json --trace-glob 'tests/golden/*.jsonl'
cargo fmt --check -p nautilus-cli
cargo clippy -p nautilus-cli --all-targets -- -D warnings
scripts/ai/verify_fast.sh
git diff --check
```

`scripts/ai/verify_release_strict.sh v19` is intentionally not a required gate
until the strict provenance verifier grows v19 support. The v0.19 release gate
uses `v19-release-gates` as the authoritative local/offline release bundle.

## Evidence

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V190-001.md
docs/rust-cutover/evidence/V190-002.md
docs/rust-cutover/evidence/V190-003.md
docs/rust-cutover/evidence/V190-004.md
docs/rust-cutover/evidence/V190-005.md
docs/rust-cutover/evidence/V190-006.md
docs/rust-cutover/evidence/V190-007.md
docs/rust-cutover/evidence/V190-008.md
docs/rust-cutover/evidence/V190-009.md
docs/rust-cutover/evidence/V190-010.md
```

## Release Status

This document is the release-note candidate for the formal
`ntpro-rust-only-v0.19.0` GitHub Release. It must not be described as published
until the V190-010 PR is merged, the tag is pushed, and the release-tag workflow
passes `release-v19-release-gates`.
