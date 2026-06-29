# NTPRO Rust-only v0.20.0 Release Notes

Date: 2026-06-29
Executor: Codex
Status: RELEASED
Tag: `ntpro-rust-only-v0.20.0`
Release name: `NTPRO Rust-only v0.20.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.20.0`
Release commit: `resolved by release tag`
Published at: `resolved by GitHub Release`
Hosted release gate stage: `release-v20-release-gates`

## Summary

`v0.20.0` publishes the Owner-Approved Production Order Lifecycle Foundation.
It connects the v0.20 scope decision, safety contract, pre-submit risk gate,
owner approval lifecycle, signing-material env gate, deterministic request
builder, guarded single-shot submit candidate, redacted response evidence,
post-submit readback reconciliation, failure/no-retry evidence, Dashboard
read-only audit, executable golden traces, and strict release provenance into
one release boundary.

Plain Chinese summary: v0.20.0 是“Owner 人工批准的生产订单生命周期基础版”发布面。大白话：
它只证明一个受 owner approval 约束的 submit/readback/audit 基础链条。必须有风险检查、
owner approval、签名材料 env gate、单次 submit 证据、脱敏 response、post-submit
readback、failure/no-retry 证据、只读 Dashboard audit 和 golden traces。它不是产品级
实盘交易终端，不包含策略自动实盘、批量订单、自动撤单、retry/replace/amend/flatten、
自动补救或 Dashboard 下单控件。

## Product Claim

```text
capability = Owner-Approved Production Order Lifecycle Foundation
production submit lifecycle foundation = included
owner approval = required
risk gate = required
signing material env gate = required
single submit attempt = required
post-submit response redaction = required
post-submit readback = required
failure/no-retry evidence = required
Dashboard surface = read-only audit view
golden traces = included
aggregate release gate = v20-release-gates
strict provenance gate = scripts/ai/verify_release_strict.sh v20
Dashboard order controls = not included
implicit retry = not included
automatic cancel = not included
automatic remediation = not included
bulk order execution = not included
retry / replace / amend / flatten = not included
strategy-driven production execution = not included
general production trading platform claim = not included
```

## Changed

- Added the v0.20 scope decision and go/no-go boundary.
- Added Rust evidence for pre-submit risk, owner approval, signing-material
  env gating, deterministic request building, guarded submit candidate,
  response redaction, readback reconciliation, failure/no-retry evidence, and
  Dashboard read-only order lifecycle audit.
- Added production order lifecycle golden traces covering blocked, accepted,
  venue rejected, unknown response, readback mismatch, and readback missing
  cases.
- Added `v20-release-gates` and `v20-strict-provenance` release stages.
- Added a machine-readable v0.20 release manifest whose source, tree, binary,
  and hash fields are resolved by strict release provenance.

## Release-Blocking Conditions

The v0.20 release guard rejects the package if any of these conditions appear:

```text
v0.19.1 closeout evidence missing = release-blocking
V200 evidence missing = release-blocking
missing owner approval = release-blocking
missing risk gate = release-blocking
missing signing material env gate = release-blocking
missing guarded submit candidate evidence = release-blocking
missing response redaction = release-blocking
missing post-submit readback = release-blocking
missing failure/no-retry evidence = release-blocking
missing Dashboard read-only audit evidence = release-blocking
missing production order lifecycle golden traces = release-blocking
raw credential plaintext = release-blocking
raw venue response/readback body persistence = release-blocking
implicit retry = release-blocking
second submit attempt = release-blocking
automatic cancel or remediation = release-blocking
Dashboard order, approval, cancel, or retry controls = release-blocking
release manifest or binary provenance mismatch = release-blocking
```

## Not Included

```text
Dashboard order controls = not included
Dashboard approval controls = not included
Dashboard cancel controls = not included
Dashboard retry controls = not included
implicit retry = not included
automatic cancel = not included
automatic remediation = not included
bulk order execution = not included
multi-account production execution = not included
multi-venue production execution = not included
strategy-driven production execution = not included
retry / replace / amend / flatten = not included
listenKey lifecycle = not included
binary asset publication = not included
general production trading platform claim = not included
```

## Validation

Required release validation:

```text
scripts/ai/verify_release.sh v20-release-gates
scripts/ai/verify_release.sh v20-strict-provenance
scripts/ai/verify_release_strict.sh v20
scripts/ai/verify_v20_release_gates.sh
scripts/ai/verify_v20_order_lifecycle_golden_traces.sh
scripts/ai/run_golden_traces.sh
python3 scripts/ai/validate_golden_trace_release_scope.py --manifest docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json --trace-glob 'tests/golden/*.jsonl'
scripts/ai/check_release_surface_current.sh
scripts/ai/check_github_release_published.sh
scripts/ai/verify_fast.sh
git diff --check
```

## Evidence

Readiness evidence is recorded in:

```text
docs/rust-cutover/evidence/V200-000.md
docs/rust-cutover/evidence/V200-001.md
docs/rust-cutover/evidence/V200-002.md
docs/rust-cutover/evidence/V200-003.md
docs/rust-cutover/evidence/V200-004.md
docs/rust-cutover/evidence/V200-005.md
docs/rust-cutover/evidence/V200-006.md
docs/rust-cutover/evidence/V200-007.md
docs/rust-cutover/evidence/V200-008.md
docs/rust-cutover/evidence/V200-009.md
docs/rust-cutover/evidence/V200-010.md
docs/rust-cutover/evidence/V200-011.md
docs/rust-cutover/evidence/V200-012.md
docs/rust-cutover/release/v0_20_0_release_manifest.json
```

## Release Status

```text
release status = RELEASED
release tag = ntpro-rust-only-v0.20.0
release name = NTPRO Rust-only v0.20.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.20.0
release commit = resolved by release tag
hosted release gate stage = release-v20-release-gates
GitHub Release draft = false
GitHub Release prerelease = false
```

The release remains limited to the owner-approved production order lifecycle
foundation. It does not claim product-grade live trading, automatic execution,
retry, remediation, multi-account or multi-venue production execution, or
Dashboard operation controls.
