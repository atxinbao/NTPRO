# V200-009 Verification

Date: 2026-06-29
Executor: Codex
Task: `V200-009` / GitHub issue `#620`

## Commands

```text
cargo fmt -p nautilus-risk = PASS
cargo test -p nautilus-risk --test v20_failure_no_retry -- --nocapture = PASS, 5 passed
cargo clippy -p nautilus-risk --all-targets -- -D warnings = PASS
rg -n "ntpro\\.v200_failure_no_retry_evidence\\.v1|v200_failure_validation_failed|v200_failure_approval_failed|v200_failure_credential_unavailable|v200_failure_submit_failed|v200_failure_venue_rejected|v200_failure_response_unknown|v200_failure_readback_missing|v200_failure_readback_mismatch|v200_failure_cancel_required|v200_failure_audit_incomplete|no_implicit_retry|retry_attempted = false|automatic_remediation_allowed = false|dashboard_order_controls_enabled = false" crates/risk/src/v20_failure_no_retry.rs crates/risk/tests/v20_failure_no_retry.rs docs/rust-cutover/release/v0_20_0_failure_no_retry_evidence.md docs/rust-cutover/evidence/V200-009.md verification.md = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

V200-009 implements local Rust failure and no-retry evidence in
`nautilus-risk`. It records blocked, validation_failed, approval_failed,
credential_unavailable, submit_failed, venue_rejected, response_unknown,
readback_missing, readback_mismatch, cancel_required, and audit_incomplete
categories with stable codes, source evidence pointers, and next allowed
actions. It keeps implicit retry, replacement, amendment, flattening,
automatic cancel, automatic remediation, strategy continuation, and Dashboard
order controls disabled.

# V200-008 Verification

Date: 2026-06-29
Executor: Codex
Task: `V200-008` / GitHub issue `#619`

## Commands

```text
cargo fmt -p nautilus-risk = PASS
cargo test -p nautilus-risk --test v20_submit_readback_reconciliation -- --nocapture = PASS, 6 passed
cargo clippy -p nautilus-risk --all-targets -- -D warnings = PASS
rg -n "ntpro\\.v200_submit_readback_reconciliation\\.v1|v200_submit_readback_matched|v200_submit_readback_mismatched|v200_submit_readback_missing|v200_submit_readback_ambiguous|v200_submit_readback_failed|risk_evidence_required|cancel_or_audit_input_ready|dashboard_read_only_consumable|automatic_cancel_attempted = false" crates/risk/src/v20_submit_readback_reconciliation.rs crates/risk/tests/v20_submit_readback_reconciliation.rs docs/rust-cutover/release/v0_20_0_submit_readback_reconciliation.md docs/rust-cutover/evidence/V200-008.md verification.md = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

V200-008 implements local Rust post-submit readback reconciliation evidence in
`nautilus-risk`. It consumes local submit expectation, V200-007 redacted
response evidence, and a venue readback snapshot to classify matched,
mismatched, missing, ambiguous, readback_failed, or blocked outcomes. It creates
read-only dashboard/audit evidence and risk evidence inputs without automatic
cancel, retry, replacement, amendment, flattening, raw readback recording, or
Dashboard order controls.

# V200-007 Verification

Date: 2026-06-29
Executor: Codex
Task: `V200-007` / GitHub issue `#618`

## Commands

```text
cargo fmt -p nautilus-risk = PASS
cargo test -p nautilus-risk --test v20_submit_response_redaction -- --nocapture = PASS, 7 passed after fixing venue_status retention
cargo clippy -p nautilus-risk --all-targets -- -D warnings = PASS
rg -n "ntpro\\.v200_submit_response_redaction\\.v1|v200_submit_response_accepted|v200_submit_response_rejected|v200_submit_response_unknown|v200_submit_response_malformed|v200_submit_response_sensitive_material_observed|response_digest|raw_exchange_response_recorded|signature_material_recorded|readback_success_inferred = false|dashboard_raw_response_enabled = false" crates/risk/src/v20_submit_response_redaction.rs crates/risk/tests/v20_submit_response_redaction.rs docs/rust-cutover/release/v0_20_0_submit_response_redaction.md docs/rust-cutover/evidence/V200-007.md verification.md = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

V200-007 implements local Rust production submit response redaction evidence in
`nautilus-risk`. It consumes V200-006 submitted attempt evidence, records
accepted/rejected/unknown/malformed/blocked states, emits request and response
digests plus minimal order correlation fields, and keeps raw exchange payloads,
headers, credentials, signatures, tokens, signed query strings, signed URLs,
Dashboard raw response controls, and readback-success inference out of the
artifact.

# V200-006 Verification

Date: 2026-06-29
Executor: Codex
Task: `V200-006` / GitHub issue `#617`

## Commands

```text
cargo fmt -p nautilus-risk = PASS
cargo test -p nautilus-risk --test v20_submit_candidate -- --nocapture = PASS, 9 passed
cargo clippy -p nautilus-risk --all-targets -- -D warnings = PASS
rg -n "ntpro\\.v200_guarded_single_shot_submit_candidate\\.v1|v200_guarded_submit_submitted|v200_guarded_submit_duplicate_rejected|v200_guarded_submit_manual_gate_missing|submit_attempt_evidence_ready|owner_approval_consumed|production_submit_attempted|adapter_submit_handoff_allowed|readback_required|dashboard_order_controls_enabled = false" crates/risk/src/v20_submit_candidate.rs crates/risk/tests/v20_submit_candidate.rs docs/rust-cutover/release/v0_20_0_guarded_single_shot_submit_candidate.md docs/rust-cutover/evidence/V200-006.md verification.md = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

V200-006 implements the local Rust guarded single-shot submit candidate gate in
`nautilus-risk`. It records blocked, preview, dry-run, and submitted evidence;
submit mode requires all prerequisite evidence, matching request digest, manual
online gate, and no prior matching submit digest before consuming owner
approval.

# V200-005 Verification

Date: 2026-06-29
Executor: Codex
Task: `V200-005` / GitHub issue `#616`

## Commands

```text
cargo fmt -p nautilus-risk = PASS
cargo test -p nautilus-risk --test v20_submit_request_builder -- --nocapture = PASS, 6 passed
cargo clippy -p nautilus-risk --all-targets -- -D warnings = PASS
rg -n "ntpro\\.v200_single_shot_submit_request_builder\\.v1|v200_submit_request_built|v200_submit_request_missing_risk_allow|v200_submit_request_missing_owner_approval|v200_submit_request_missing_signing_readiness|v200_submit_request_candidate_mismatch|request_digest|redacted_preview|network_attempted = false|raw_signed_payload_persisted = false" crates/risk/src/v20_submit_request_builder.rs crates/risk/tests/v20_submit_request_builder.rs docs/rust-cutover/release/v0_20_0_single_shot_submit_request_builder.md docs/rust-cutover/evidence/V200-005.md verification.md = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

V200-005 implements a local Rust single-shot submit request builder in
`nautilus-risk`. The builder requires risk allow, active owner approval, and
signing readiness evidence before producing a deterministic request digest and
redacted preview. Missing prerequisite evidence, candidate mismatch, and
unsupported order shapes are rejected. The builder does not sign, submit, send,
retry, persist raw request payloads, or enable Dashboard order controls.

# V200-004 Verification

Date: 2026-06-29
Executor: Codex
Task: `V200-004` / GitHub issue `#615`

## Commands

```text
cargo fmt -p nautilus-risk = PASS
cargo test -p nautilus-risk --test v20_signing_material_gate -- --nocapture = PASS, 6 passed
cargo clippy -p nautilus-risk --all-targets -- -D warnings = PASS
rg -n "ntpro\\.v200_signing_material_env_gate\\.v1|v200_signing_material_ready|v200_signing_material_environment_mismatch|v200_signing_material_missing|v200_signing_material_empty|v200_signing_material_source_not_env|raw_secret_persisted|dashboard_credential_output_enabled|ntpro-fnv64" crates/risk/src/v20_signing_material_gate.rs crates/risk/tests/v20_signing_material_gate.rs docs/rust-cutover/release/v0_20_0_signing_material_env_gate.md docs/rust-cutover/evidence/V200-004.md verification.md = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

V200-004 implements a local Rust env-only signing material gate in
`nautilus-risk`. The gate blocks missing env material, empty material,
environment mismatch, and non-env sources with stable evidence codes. Evidence
and redacted artifacts include only env var names, material kind, source,
presence, and non-secret fingerprints; they do not include raw key, secret,
token, or signature material and do not add Dashboard credential controls.

# V200-003 Verification

Date: 2026-06-29
Executor: Codex
Task: `V200-003` / GitHub issue `#614`

## Commands

```text
cargo fmt -p nautilus-risk = PASS
cargo test -p nautilus-risk --test v20_owner_approval -- --nocapture = PASS, 10 passed
cargo clippy -p nautilus-risk --all-targets -- -D warnings = PASS
rg -n "ntpro\\.v200_owner_approval_lifecycle_event\\.v1|v200_owner_approval_expired|v200_owner_approval_revoked|v200_owner_approval_already_consumed|v200_owner_approval_request_digest_mismatch|v200_owner_approval_environment_mismatch|submit_consumption_allowed|dashboard_approval_controls_enabled|approval_reusable = false" crates/risk/src/v20_owner_approval.rs crates/risk/tests/v20_owner_approval.rs docs/rust-cutover/release/v0_20_0_owner_approval_lifecycle.md docs/rust-cutover/evidence/V200-003.md verification.md = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

V200-003 implements a local Rust owner approval lifecycle in `nautilus-risk`.
The lifecycle binds approval request digest, production order scope, owner,
expiry, nonce, environment, and release provenance to one submit candidate.
Expired, revoked, owner-rejected, already-consumed, digest-mismatched, and
cross-environment approval reuse paths all produce evidence and cannot be
consumed for submit. It does not add Dashboard approval controls, adapter
submit calls, retry, or automatic remediation.

# V200-002 Verification

Date: 2026-06-29
Executor: Codex
Task: `V200-002` / GitHub issue `#613`

## Commands

```text
cargo fmt -p nautilus-risk = PASS
cargo test -p nautilus-risk --test v20_pre_submit_gate -- --nocapture = PASS, 10 passed
cargo clippy -p nautilus-risk --all-targets -- -D warnings = PASS
rg -n "ntpro\\.v200_pre_submit_risk_gate_decision\\.v1|v200_pre_submit_unknown_field|v200_pre_submit_account_unknown|v200_pre_submit_price_missing|v200_pre_submit_notional_limit_exceeded|v200_pre_submit_approval_expired|v200_pre_submit_environment_mismatch|v200_pre_submit_provenance_missing|production_order_submission_allowed|dashboard_order_controls_enabled" crates/risk/src/v20_pre_submit_gate.rs crates/risk/tests/v20_pre_submit_gate.rs docs/rust-cutover/release/v0_20_0_pre_submit_risk_gate.md docs/rust-cutover/evidence/V200-002.md verification.md = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

V200-002 implements a local Rust pre-submit risk gate in `nautilus-risk`. The
gate returns allow, deny, or blocked evidence with stable codes for unknown
fields, unknown account/venue, missing fields, limit breaches, missing or
expired approval, environment mismatch, and missing or invalid release
provenance. It does not submit orders, connect to adapters, retry, remediate,
or enable Dashboard order controls.

# V200-001 Verification

Date: 2026-06-29
Executor: Codex
Task: `V200-001` / GitHub issue `#612`

## Commands

```text
rg -n "draft|risk_checked|owner_approved|submit_attempted|readback_verified|cancel_requested|cancel_verified|audit_closed|approval_expired|readback_mismatch|cancel_failed|retry_attempted = false|dashboard_order_controls_enabled = false|Silent failure is forbidden|ntpro.v200_order_lifecycle_safety_contract.v1" docs/rust-cutover/release/v0_20_0_order_lifecycle_safety_contract.md docs/rust-cutover/evidence/V200-001.md = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

V200-001 defines the v0.20 production order lifecycle safety contract and state
machine before implementation. The contract covers normal path, rejection,
expired approval, reused/mismatched approval, readback mismatch, cancel failure,
no implicit retry, audit-first evidence, and read-only Dashboard boundaries.

# V200-000 Verification

Date: 2026-06-29
Executor: Codex
Task: `V200-000` / GitHub issue `#611`

## Commands

```text
gh issue view 604 --repo atxinbao/NTPRO --json number,title,state,closedAt,url = PASS, CLOSED at 2026-06-28T12:50:56Z
gh issue view 605 --repo atxinbao/NTPRO --json number,title,state,closedAt,url = PASS, CLOSED at 2026-06-28T13:22:11Z
gh issue view 606 --repo atxinbao/NTPRO --json number,title,state,closedAt,url = PASS, CLOSED at 2026-06-29T00:22:02Z
gh issue view 607 --repo atxinbao/NTPRO --json number,title,state,closedAt,url = PASS, CLOSED at 2026-06-29T08:08:57Z
gh issue view 608 --repo atxinbao/NTPRO --json number,title,state,closedAt,url = PASS, CLOSED at 2026-06-29T08:58:35Z
gh issue view 609 --repo atxinbao/NTPRO --json number,title,state,closedAt,url = PASS, CLOSED at 2026-06-29T09:33:55Z
gh issue view 610 --repo atxinbao/NTPRO --json number,title,state,closedAt,url = PASS, CLOSED at 2026-06-29T10:19:08Z
test -f docs/rust-cutover/release/v0_19_1_release_notes.md = PASS
test -f docs/rust-cutover/release/v0_19_1_readiness_report.md = PASS
test -f docs/rust-cutover/evidence/V191-007.md = PASS
rg -n "SD-003|go_for_v200_planning = true|go_for_general_production_trading_platform = false|strategy-driven production execution|Dashboard order button|V200-001 through V200-012|#604|#610|CLOSEOUT EVIDENCE COMPLETE" docs/rust-cutover/scope docs/rust-cutover/evidence/V200-000.md docs/rust-cutover/release verification.md = PASS
stale v0.19.1 pending marker scan = PASS, no matches
scripts/ai/verify_release.sh release-surface-current-guard = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

V200-000 records a GO decision for bounded owner-approved production order
lifecycle foundation work after this PR merges. The decision remains NO-GO for
strategy-driven production execution, automatic order placement, bulk orders,
MARKET orders without later explicit approval, multi-account/multi-venue
execution, retry/replace/amend/flatten, automatic remediation, Dashboard order
or approval buttons, and any general production trading platform claim.

# V191-007 Verification

Date: 2026-06-29
Executor: Codex
Task: `V191-007` / GitHub issue `#610`

## Commands

```text
bash -n scripts/ai/verify_v19_release_gates.sh scripts/ai/verify_release.sh scripts/ai/verify_release_strict.sh = PASS
NTPRO_V19_SKIP_BUILD=1 NTPRO_V19_NAUTILUS_BIN="$PWD/target/debug/nautilus" scripts/ai/verify_v19_release_gates.sh = EXPECTED FAIL, debug binary rejected before any release-evidence-looking output
scripts/ai/verify_v19_release_gates.sh = PASS, printed release binary: /Users/mac/Documents/NTPRO/target/release/nautilus; checked 27 v190 artifacts and 10 actual-cancel trace cases
scripts/ai/verify_release.sh v19-release-gates = PASS
scripts/ai/verify_release_strict.sh v19 = PASS, strict_release_provenance status=ok, binary_path=/Users/mac/Documents/NTPRO/target/release/nautilus
rg -n "debug/nautilus|target/release/nautilus|local smoke only|release binary|v19-release-gates" scripts/ai docs/rust-cutover/release docs/rust-cutover/evidence verification.md = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

The standalone v19 release gate now defaults to `target/release/nautilus` and
builds the release binary when no explicit binary is supplied. Explicit
`target/debug/nautilus` use is rejected by default; non-release runs must opt in
with `NTPRO_V19_ALLOW_LOCAL_SMOKE_ONLY=1` and are labeled `local smoke only`.

This verification keeps `scripts/ai/verify_release.sh v19-release-gates` as the
authoritative release dispatcher path and keeps `scripts/ai/verify_release_strict.sh v19`
as the strict provenance path. It does not open production network/env gates,
production order submission, automatic cancel, bulk cancel, retry, second
cancel, remediation, Dashboard execution, or Dashboard cancel controls.

# V191-006 Verification

Date: 2026-06-29
Executor: Codex
Task: `V191-006` / GitHub issue `#609`

## Commands

```text
gh pr view 598 --repo atxinbao/NTPRO --json number,title,state,mergedAt,reviews,reviewDecision,url = PASS, reviews=[]
gh issue view 581 --repo atxinbao/NTPRO --json number,title,state,closedAt,url = PASS, state=CLOSED
rg -n "V190-004|PR #598|post-merge review|REVIEW_REQUIRED|owner approval consumption|no retry|no bulk|no second cancel|Dashboard" docs/rust-cutover/evidence docs/rust-cutover/release = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

The V190-004 / PR #598 review-record gap is now captured as explicit v0.19.1
closeout evidence. PR #598 is merged and issue #581 is closed, but GitHub
review submissions are absent, so `V191-006_actual_cancel_review_attestation.md`
records a compensating post-merge review of the actual-cancel command boundary,
manual-online env gates, owner approval consumption, one-order/one-venue/
one-attempt limit, readback/failure-evidence follow-up, and no retry / no bulk
/ no second cancel / no Dashboard execution boundary.

This verification does not change runtime behavior, CLI behavior, adapter
behavior, release tags, release assets, order submission, automatic cancel,
bulk cancel, retry, second cancel, remediation, Dashboard order controls, or
Dashboard cancel controls.

# V191-005 Verification

Date: 2026-06-29
Executor: Codex
Task: `V191-005` / GitHub issue `#608`

## Commands

```text
bash -n scripts/ai/verify_release_strict.sh scripts/ai/verify_release.sh = PASS
git diff --check = PASS
rg -n "candidate|pending|RELEASE CANDIDATE" docs/rust-cutover/release/v0_19_0_release_notes.md docs/rust-cutover/release/v0_19_0_readiness_report.md = PASS, no matches
NTPRO_RELEASE_GATE=1 NTPRO_RELEASE_STRICT_SKIP_BUILD=1 scripts/ai/verify_release_strict.sh v19 = EXPECTED FAIL, dirty tracked worktree rejected
NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 NTPRO_RELEASE_STRICT_SKIP_BUILD=1 scripts/ai/verify_release_strict.sh v19 = EXPECTED FAIL, HEAD/tag mismatch rejected
NTPRO_V19_RELEASE_MANIFEST=/tmp/ntpro-v191-005-missing-manifest.json NTPRO_RELEASE_STRICT_SKIP_BUILD=1 scripts/ai/verify_release_strict.sh v19 = EXPECTED FAIL, missing release manifest rejected
NTPRO_V19_RELEASE_NOTES=<temp stale notes> NTPRO_RELEASE_STRICT_SKIP_BUILD=1 scripts/ai/verify_release_strict.sh v19 = EXPECTED FAIL, stale release status rejected
NTPRO_RELEASE_STRICT_SKIP_BUILD=1 NTPRO_RELEASE_STRICT_SKIP_V19_GATES=1 scripts/ai/verify_release_strict.sh v19 = PASS
scripts/ai/verify_release_strict.sh v19 = PASS
scripts/ai/verify_release.sh v19-strict-provenance = PASS
NTPRO_RELEASE_STRICT_VERIFY_ONLY=1 scripts/ai/verify_release_strict.sh v19 = PASS
NTPRO_RELEASE_GATE=1 scripts/ai/verify_release_strict.sh v19 = PASS after commit clean-tree, source_dirty=false
bash -n scripts/ai/verify_release_strict.sh scripts/ai/verify_release.sh scripts/ai/verify_fast.sh = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

The strict release provenance verifier now supports `v19`, validates the
published `ntpro-rust-only-v0.19.0` release notes/readiness/manifest, records
the source release tag commit/tree, current source commit/tree, cargo/rustc
versions, release binary hash/bytes/version, release manifest hash, golden trace
manifest hash, and v19 gate output root, and writes the machine-readable strict
manifest under `target/ntpro-v190/`. The aggregate `verify_release.sh` command
now exposes `v19-strict-provenance`.

This verification does not change the v0.19.0 release tag, runtime behavior,
adapter behavior, production order submission, automatic cancel, bulk cancel,
retry, second cancel, remediation, Dashboard order controls, Dashboard cancel
controls, or release asset publication.

# V191-004 Verification

Date: 2026-06-29
Executor: Codex
Task: `V191-004` / GitHub issue `#607`

## Commands

```text
scripts/ai/check_github_release_published.sh = PASS
NTPRO_CURRENT_RELEASE_VERSION=v0.19.0 NTPRO_CURRENT_RELEASE_TAG=ntpro-rust-only-v0.19.0 NTPRO_CURRENT_RELEASE_NAME='NTPRO Rust-only v0.19.0' scripts/ai/check_github_release_published.sh = PASS
scripts/ai/verify_release.sh release-publication-guard = PASS
bash -n scripts/ai/check_github_release_published.sh scripts/ai/verify_release.sh = PASS
NTPRO_CURRENT_RELEASE_VERSION=v0.18.0 NTPRO_CURRENT_RELEASE_TAG=ntpro-rust-only-v0.18.0 NTPRO_CURRENT_RELEASE_NAME='NTPRO Rust-only v0.18.0' scripts/ai/check_github_release_published.sh = PASS
rg -n 'Status: RELEASED|Tag: `ntpro-rust-only-v0\\.19\\.0`|Owner-Approved Single-Shot Actual Cancel|actual cancel only owner-approved single-shot|owner approval = required|single order = required|single venue = required|single execution attempt = required|post-cancel readback = required|failure evidence = required|Dashboard cancel button = not included|production order submit lifecycle = not included|automatic cancel = not included|bulk cancel = not included|retry / replace / amend / flatten = not included' docs/rust-cutover/release/v0_19_0_release_notes.md = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

The GitHub Release publication guard now defaults to `v0.19.0`, validates the
formal `ntpro-rust-only-v0.19.0` GitHub Release and local release notes, and
retains explicit `v0.18.0` support. The live v0.19.0 GitHub Release body was
aligned to `docs/rust-cutover/release/v0_19_0_release_notes.md` so the guard can
check both local and remote publication metadata against the same formal field
family. This verification does not add production order submission, automatic
cancel, bulk cancel, retry, second cancel, remediation, Dashboard order
controls, Dashboard cancel controls, runtime behavior, adapter behavior,
release tag changes, or release asset changes.

# V191-003 Verification

Date: 2026-06-28
Executor: Codex
Task: `V191-003` / GitHub issue `#606`

## Commands

```text
scripts/ai/check_release_surface_current.sh = PASS
NTPRO_CURRENT_RELEASE_VERSION=v0.19.0 NTPRO_CURRENT_RELEASE_TAG=ntpro-rust-only-v0.19.0 NTPRO_NEXT_PATCH_VERSION=v0.19.1 NTPRO_NEXT_CAPABILITY_VERSION=v0.20.0 NTPRO_CURRENT_RELEASE_CAPABILITY='Owner-Approved Single-Shot Actual Cancel' scripts/ai/check_release_surface_current.sh = PASS
rg -n "Current source tag|Latest formal release|next patch|next capability|v0\\.18\\.0|v0\\.19\\.0|v0\\.19\\.1|v0\\.20\\.0" README.md ROADMAP.md docs/versioning.md docs/rust-cutover/release/README.md = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

The public current release surface now points to the formal
`ntpro-rust-only-v0.19.0` release and Owner-Approved Single-Shot Actual Cancel
capability. README, ROADMAP, versioning docs, the release index, and the default
current-release guard agree that `v0.19.1` is only the actual-cancel release
closeout / provenance hardening patch and `v0.20.0` is the next capability
track for Owner-Approved Production Order Lifecycle Foundation. This
verification does not add production order submission, automatic cancel, bulk
cancel, retry, second cancel, remediation, Dashboard order controls, or
Dashboard cancel controls.

# V190-009 Verification

Date: 2026-06-28
Executor: Codex
Task: `V190-009` / GitHub issue `#585`

## Commands

```text
cargo fmt -p nautilus-cli = PASS
python3 scripts/ai/golden_trace_runner.py tests/golden/actual_cancel_schema.jsonl --mode validate-only = PASS, 10 rows
cargo test -p nautilus-cli --test golden_trace_actual_cancel = PASS, 1 test
TRACE_GLOB=tests/golden/actual_cancel_schema.jsonl RUN_RUST_GOLDEN_TRACE_HARNESS=0 RUN_RUST_MARKET_DATA_TRACE_REPLAY=0 RUN_RUST_CACHE_MSGBUS_TRACE_REPLAY=0 RUN_RUST_BACKTEST_TRACE_REPLAY=0 RUN_RUST_BACKTEST_LIVE_PARITY_TRACE_REPLAY=0 RUN_RUST_LIVE_SANDBOX_TRACE_REPLAY=0 RUN_RUST_ORDER_LIFECYCLE_TRACE_REPLAY=0 RUN_RUST_RISK_REJECTION_TRACE_REPLAY=0 RUN_RUST_ADAPTER_PAYLOAD_TRACE_REPLAY=0 RUN_RUST_LIVE_ALPHA_RECONCILIATION_TRACE_REPLAY=0 RUN_RUST_LIVE_ALPHA_MUTATION_DRY_RUN_TRACE_REPLAY=0 RUN_RUST_ACTUAL_CANCEL_TRACE_REPLAY=1 scripts/ai/run_golden_traces.sh = PASS
scripts/ai/verify_v19_actual_cancel_golden_traces.sh = PASS
scripts/ai/run_golden_traces.sh = PASS
python3 scripts/ai/validate_golden_trace_release_scope.py --manifest docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json --trace-glob 'tests/golden/*.jsonl' = PASS, 45 cases, 44 executable replay, 1 schema-only scoped
scripts/ai/verify_release.sh v19-release-gates = UNAVAILABLE, unknown verify_release stage: v19-release-gates
cargo fmt --check -p nautilus-cli = PASS
cargo clippy -p nautilus-cli --all-targets -- -D warnings = PASS
scripts/ai/verify_fast.sh = PASS
git diff --check = PASS
PR smoke classifier simulation for this change set = PASS
  v19_smoke = true
  heavy_rust = true
  heavy_rust_reason = verification.md
```

## Result

The v0.19 actual-cancel golden trace coverage is implemented as a local/offline
fixture and Rust harness. It covers success, approval missing, approval reused,
risk mismatch, adapter unsupported, cancel rejected, timeout, unknown, already
cancelled, and partial fill. Each case carries request, response, readback,
audit, and provenance references. Partial-fill quantity fields remain decimal
strings, and no quantity/price arithmetic path changed. This verification does
not add live venue credentials, live broker connectivity, actual-cancel runtime
behavior changes, production order submit expansion, retry, remediation, second
cancel, Dashboard cancel controls, raw response persistence, or credential
persistence. The `v19-release-gates` stage remains unavailable and is owned by
V190-010.

# V190-008 Verification

Date: 2026-06-28
Executor: Codex
Task: `V190-008` / GitHub issue `#584`

## Commands

```text
cargo fmt -p nautilus-cli = PASS
cargo test -p nautilus-cli production_actual_cancel_audit --lib = PASS, 6 tests
cargo test -p nautilus-cli production_cancel_recovery --lib = PASS, 7 tests
scripts/ai/verify_v19_dashboard_actual_cancel_audit_view.sh = PASS
cargo fmt --check -p nautilus-cli = PASS
cargo clippy -p nautilus-cli --all-targets -- -D warnings = PASS
scripts/ai/verify_fast.sh = PASS
rg -n "cancel button|approve button|retry|bulk|actual_cancel|read-only|readonly" crates docs scripts = PASS
git diff --check = PASS
```

## Result

The v0.19 Dashboard actual-cancel audit view is implemented as a read-only
local artifact surface. It consumes risk gate, owner approval, single-shot
cancel attempt, post-cancel readback reconciliation, and failure evidence
artifacts. The view distinguishes `ready`, `recovered`, `degraded`, `failed`,
and `unknown`; missing evidence, schema mismatch, provenance mismatch, stale
evidence, unknown readback, source issues, and Dashboard/control boundary
violations do not render as healthy/recovered. This verification does not add a
Dashboard cancel button, owner approval button, retry button, bulk action,
trader terminal, Dashboard write operation, or multi-account/multi-strategy
view.

# V190-007 Verification

Date: 2026-06-28
Executor: Codex
Task: `V190-007` / GitHub issue `#583`

## Commands

```text
cargo fmt -p nautilus-cli = PASS
cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_failure_evidence_options --lib = PASS, 1 test
cargo test -p nautilus-cli actual_cancel_failure_evidence --lib = PASS, 3 tests
cargo test -p nautilus-cli actual_cancel --lib = PASS, 18 tests
scripts/ai/verify_v19_actual_cancel_failure_evidence.sh = PASS
cargo fmt --check -p nautilus-cli = PASS
cargo clippy -p nautilus-cli --all-targets -- -D warnings = PASS
scripts/ai/verify_fast.sh = PASS
rg -n "failure evidence|partial-success|partial fill|venue_unavailable|adapter_failure|unknown_not_recovered|production-mutation-actual-cancel-failure-evidence" crates/cli/src docs/rust-cutover scripts/ai verification.md = PASS
git diff --check = PASS
```

## Result

The v0.19 actual-cancel failure and partial-success evidence command is
implemented as a local/offline evidence command. It consumes V190-006 readback
reconciliation plus request, response, readback, and audit refs, and classifies
`cancel_confirmed`, `already_cancelled`, `rejected`, `timeout`, `unknown`,
`partial_fill`, `filled_before_cancel`, `venue_unavailable`, and
`adapter_failure`. Unknown outcomes are never marked recovered. Partial-fill
outcomes expose residual risk and require manual review. The artifact is
Dashboard/release-gate consumable and does not enable retry, remediation,
compensation trades, second cancel, network readback, raw persistence, or
Dashboard order/cancel controls.

# V190-006 Verification

Date: 2026-06-28
Executor: Codex
Task: `V190-006` / GitHub issue `#582`

## Commands

```text
cargo fmt -p nautilus-cli = PASS
cargo test -p nautilus-cli actual_cancel --lib = PASS, 15 tests
cargo test -p nautilus-cli actual_cancel_readback_reconciliation --lib = PASS, 3 tests
cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_readback_reconciliation_options --lib = PASS, 1 test
scripts/ai/verify_v19_post_cancel_readback_reconciliation.sh = PASS
cargo fmt --check -p nautilus-cli = PASS
cargo clippy -p nautilus-cli --all-targets -- -D warnings = PASS
scripts/ai/verify_fast.sh = PASS
rg -n "readback|reconciliation|partial fill|unknown|already cancelled" crates docs scripts = PASS
git diff --check = PASS
```

## Result

The v0.19 actual-cancel post-readback reconciliation command is implemented as
a local/offline evidence command. It requires a recorded V190-004 actual cancel
attempt with `readback_required=true`, consumes redacted readback metadata, and
classifies `cancel_confirmed`, `already_cancelled`, `filled_before_cancel`,
`unknown`, `timeout`, and `inconsistent`. Unknown, timeout, partial-fill, and
inconsistent outcomes are explicit degraded/error states and do not enable
retry, remediation, second cancel, network readback, raw persistence, or
Dashboard cancel controls. The artifact records order status, execution/fill
status, remaining quantity state, residual risk, local audit state, and
Dashboard read-only audit readiness.

# V190-004 Verification

Date: 2026-06-27
Executor: Codex
Task: `V190-004` / GitHub issue `#581`

## Commands

```text
cargo fmt -p nautilus-cli = PASS
cargo test -p nautilus-cli actual_cancel --lib = PASS, 12 tests, including persisted used marker and same-approval second-run block
cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_single_shot_options --lib = PASS, 1 test
cargo fmt --check -p nautilus-cli = PASS
cargo clippy -p nautilus-cli --all-targets -- -D warnings = PASS
scripts/ai/verify_fast.sh = PASS
rg -n "actual_cancel_single_shot|production-mutation-actual-cancel-single-shot|actual_cancel_attempt_recorded|ready_actual_cancel_command_offline_no_send|owner_approval_reused|order_identity_mismatch|blocked_missing_manual_online_gate|approval_used_after_actual_cancel_attempt|consumed_actual_cancel_request_id|consumed_by_actual_cancel_run_id" crates/cli/src docs/rust-cutover verification.md = PASS
scripts/ai/verify_release.sh v19-release-gates = UNAVAILABLE, unknown verify_release stage: v19-release-gates
git diff --check = PASS
```

## Result

The v0.19 single-shot actual cancel command is implemented as a default-offline
CLI command. It records ready/no-send evidence without `--manual-online`, and
records exactly one injected executor cancel attempt when all owner approval,
risk gate, release provenance, adapter boundary/capability, owner-supplied
order identity, CLI confirmations, and manual online env gates match. The
attempt path atomically marks the source owner approval lifecycle as `used`
before the send, records post-attempt readback requirements on the consumed
lifecycle artifact, and blocks a second run with the same approval before any
executor call. Missing gates, missing manual-online API credentials, release
mismatch, reused owner approval, unsupported adapter capability, and order
identity mismatch fail closed before any send. The artifact does not
persist raw order ids, API key values, API secret values, API key headers,
signatures, signed queries, signed URLs, request bodies, response bodies, or
response headers. This verification does not add Dashboard cancel controls,
automatic cancel, bulk/cancel-all, retry, replace, amend, flatten, remediation,
multi-account/strategy/venue cancel, or production order submit lifecycle.

# V190-005 Verification

Date: 2026-06-27
Executor: Codex
Task: `V190-005` / GitHub issue `#580`

## Commands

```text
cargo fmt -p nautilus-cli = PASS
cargo test -p nautilus-cli actual_cancel --lib = PASS, 8 tests
cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_executor_adapter_boundary_options --lib = PASS, 1 test
cargo fmt --check -p nautilus-cli = PASS
cargo clippy -p nautilus-cli --all-targets -- -D warnings = PASS
scripts/ai/verify_fast.sh = PASS
rg -n "actual_cancel_executor_adapter_boundary|adapter_venue_unsupported|single_order_cancel_request_v1|production-mutation-actual-cancel-executor-adapter-boundary" crates/cli/src docs/rust-cutover verification.md = PASS
git diff --check = PASS
```

## Result

The v0.19 cancel executor adapter boundary is implemented as a local/offline
evidence command. It records that a future actual cancel command may only use a
matched V190-003 owner approval lifecycle and adapter capability declaration
for one order, one venue, one order-id type, and one attempt. It records
request, response, post-cancel readback, audit, and adapter failure taxonomy
contracts, and fail-closes missing CLI gates, unapproved owner lifecycle,
unsupported actual cancel capability, unsupported venue, unsupported order-id
type, bulk/cancel-all, retry, automatic cancel, multi-venue, and Dashboard
execution paths. This verification does not add a network cancel send,
production adapter integration, Dashboard operation controls, automatic or bulk
cancel, retry/replace/amend/flatten/remediation, credential persistence, or
production order submit lifecycle.

# V190-003 Verification

Date: 2026-06-27
Executor: Codex
Task: `V190-003` / GitHub issue `#579`

## Commands

```text
cargo fmt -p nautilus-cli = PASS
cargo test -p nautilus-cli owner_approval --lib = PASS, 8 tests
cargo test -p nautilus-cli parses_live_production_mutation_actual_cancel_owner_approval_lifecycle_options --lib = PASS, 1 test
cargo fmt --check -p nautilus-cli = PASS
cargo clippy -p nautilus-cli --all-targets -- -D warnings = PASS
scripts/ai/verify_fast.sh = PASS
rg -n "actual_cancel_owner_approval_lifecycle|owner_approval_reused|approval_execution_authorized|production-mutation-actual-cancel-owner-approval-lifecycle" crates/cli/src docs/rust-cutover verification.md = PASS
git diff --check = PASS
```

## Result

The v0.19 owner approval execution lifecycle is implemented as a local/offline
evidence command. It authorizes one future actual cancel attempt only for a
matched, unexpired, unused owner-approved lifecycle bound to the V190-002 safety
contract, release manifest, cancel risk gate, order lineage, symbol, account
label, and venue. Missing, expired, reused, rejected, audited, release-mismatch,
order-mismatch, and missing-confirmation paths fail closed. This verification
does not add a cancel executor, adapter behavior change, network cancel
request, Dashboard approve/cancel control, automatic or bulk cancel, retry,
replace, amend, flatten, or production order submit lifecycle.

# V191-002 Verification

Date: 2026-06-28
Executor: Codex
Task: `V191-002` / GitHub issue `#605`

## Commands

```text
gh release view ntpro-rust-only-v0.18.1 --repo atxinbao/NTPRO --json tagName,name,isDraft,isPrerelease,publishedAt,targetCommitish,url = PASS
git rev-list -n1 ntpro-rust-only-v0.18.1 = c395e71960255fefbf4100654fd53ce2bf33a08f
jq empty docs/rust-cutover/release/v0_18_1_release_manifest.json = PASS
rg -n "draft_not_published|DRAFT_NOT_PUBLISHED|not_published|actual_tag.*null|no v0.18.1 tag|not publish" docs/rust-cutover/release/v0_18_1_release_manifest.json docs/rust-cutover/release/v0_18_1_release_notes.md = no matches
published release markers and no-actual-cancel boundary markers = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

The v0.18.1 prerequisite release evidence now matches the live GitHub Release
and tag. The manifest records the actual tag, release URL, release commit,
publication time, draft/prerelease flags, and resolved source commit/tree while
preserving the release/provenance-hardening-only boundary: no actual cancel
send, no automatic cancel, no automatic remediation, no Dashboard cancel
controls, no production order mutation, and no binary asset publication.

# V190-002 Verification

Date: 2026-06-27
Executor: Codex
Task: `V190-002` / GitHub issue `#578`

## Commands

```text
rg -n "single-shot|owner-approved|actual cancel|bulk|retry|Dashboard" docs crates scripts = PASS
rg -n "v0_19_0_actual_cancel_safety_contract|missing_owner_approval|owner_approval_reused|adapter_capability_missing|dashboard_operation_requested" README.md docs/rust-cutover/release docs/rust-cutover/evidence docs/versioning.md verification.md = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

The v0.19 actual cancel safety contract is documented and indexed. It binds any
future actual cancel path to one manual owner approval, one order, one venue,
one execution attempt, required owner-approval/risk-gate/order/release-manifest
and adapter-capability artifacts, and fail-closed behavior for missing, expired,
reused, stale, or mismatched evidence. This verification does not add a cancel
executor, adapter behavior change, runtime network request, production order
submit lifecycle, automatic or bulk cancel, retry/replace/amend/flatten, or
Dashboard operation controls.

# V190-001 Verification

Date: 2026-06-27
Executor: Codex
Task: `V190-001` / GitHub issue `#577`

## Commands

```text
gh issue list --repo atxinbao/NTPRO --milestone v0.18.1 --state open = PASS, no open issues
NTPRO_RELEASE_GATE=1 scripts/ai/verify_release.sh release-surface-current-guard release-publication-guard = PASS
NTPRO_RELEASE_GATE=1 scripts/ai/verify_release_strict.sh v18 = PASS
gh run view 28299860864 --repo atxinbao/NTPRO = PASS, completed success
gh release view ntpro-rust-only-v0.18.1 --repo atxinbao/NTPRO = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

The v0.19 readiness gate is open for owner-approved single-shot actual cancel
design work only. The v0.18.1 blocker issues are closed, the v0.18.1 tag and
GitHub Release are published, hosted release gate run `28299860864` succeeded
with 50 jobs and 0 failures, and local strict provenance resolves the source to
commit `c395e71960255fefbf4100654fd53ce2bf33a08f`. This verification does not
add actual cancel send, adapter changes, runtime behavior changes, production
order submission, or Dashboard cancel controls.

# V181-007 Verification

Date: 2026-06-27
Executor: Codex
Task: `V181-007` / GitHub issue `#576`

## Commands

```text
cargo test -p nautilus-cli dashboard --lib = PASS, 63 tests
cargo test -p nautilus-cli production_cancel_recovery --lib = PASS, 7 tests
cargo clippy --workspace --lib --tests --features "arrow,ffi,high-precision,streaming,defi" -- -D warnings = PASS
scripts/ai/verify_v18_dashboard_cancel_recovery_panel.sh = PASS
scripts/ai/verify_release.sh v18-release-gates = PASS
bash -n scripts/ai/verify_v18_dashboard_cancel_recovery_panel.sh = PASS
rg -n "actual_cancel_send_allowed|cancel_attempted|network_cancel_endpoint_attempted|schema mismatch|source_commit|source_release_tag|release_tag|dashboard_auto_approval|dashboard_cancel_controls_enabled" crates/cli/src/dashboard.rs scripts/ai/verify_v18_dashboard_cancel_recovery_panel.sh docs/rust-cutover/release docs/rust-cutover/evidence = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

Dashboard cancel recovery diagnostics now degrade the v0.18 read-only panel for
missing artifacts, schema mismatches, source commit/tag provenance mismatches,
stale artifacts, true forbidden cancel flags, and true Dashboard cancel or
auto-approval controls. No actual cancel entrypoint, owner approval write
surface, network cancel endpoint, or adapter behavior was added.

# V181-006 Verification

Date: 2026-06-27
Executor: Codex
Task: `V181-006` / GitHub issue `#575`

## Commands

```text
jq empty docs/rust-cutover/release/v0_18_1_release_manifest.json = PASS
scripts/ai/verify_release_strict.sh v18 = PASS
NTPRO_RELEASE_STRICT_VERIFY_ONLY=1 scripts/ai/verify_release_strict.sh v18 = PASS
bash -n scripts/ai/verify_release_strict.sh scripts/ai/verify_release.sh = PASS
rg -n "release manifest|manifest|v0.18.1|actual_cancel" docs/rust-cutover/release scripts/ai = PASS
git diff --check = PASS
```

## Result

The v0.18.1 release manifest is machine-readable JSON and records the v0.18.0
baseline release, planned and actual patch tag fields, release gate list,
source/binary provenance handoff, capability boundary, and no-actual-cancel
flags. `verify_release_strict.sh v18` now reads this docs manifest, validates
its key fields, and embeds the manifest path/sha256 into the generated strict
binary provenance manifest.

# V181-005 Verification

Date: 2026-06-27
Executor: Codex
Task: `V181-005` / GitHub issue `#574`

## Commands

```text
scripts/ai/verify_fast.sh = PASS
scripts/ai/verify_release.sh v18-release-gates = PASS
bash -n scripts/ai/verify_fast.sh scripts/ai/verify_release.sh scripts/ai/verify_release_strict.sh = PASS
rg -n "verify_fast|verify_release|verify_release_strict|release gate" README.md docs scripts/ai verification.md = PASS
old misleading wording scan = no matches
git diff --check = PASS
```

## Result

Default `verify_fast.sh` is now documented and printed as fast smoke only. It
checks the pinned Rust toolchain and `cargo fmt --check` by default, and it is
not release validation or release evidence. v0.18/v0.18.1 release evidence
points to `verify_release.sh`; v0.18.1 strict provenance points to
`verify_release_strict.sh`.

# V191-001 Verification

Date: 2026-06-28
Executor: Codex
Task: `V191-001` / GitHub issue `#604`

## Commands

```text
gh release view ntpro-rust-only-v0.19.0 --repo atxinbao/NTPRO --json tagName,name,isDraft,isPrerelease,publishedAt,targetCommitish,url = PASS
gh run view 28314859483 --repo atxinbao/NTPRO = PASS, completed success, 51 jobs, 0 failures
rg -n "RELEASE CANDIDATE|pending V190-010|tag = pending|GitHub Release = pending|hosted release gate = pending" docs/rust-cutover/release/v0_19_0_release_notes.md docs/rust-cutover/release/v0_19_0_readiness_report.md docs/rust-cutover/evidence/V190-010.md = no matches
published release markers, release commit, hosted gate URL, and actual-cancel-only boundary markers = PASS
git diff --check = PASS
scripts/ai/verify_fast.sh = PASS
```

## Result

The v0.19.0 release surface is closed out against the live GitHub Release and
hosted tag workflow. The closeout records the tag, release URL, release commit,
publication time, hosted `release-v19-release-gates` PASS state, merged PR, and
issue closure evidence. The release remains limited to owner-approved
single-shot actual cancel: no production order submit lifecycle, no automatic
or bulk cancel, no Dashboard cancel controls, no retry, no second cancel, and
no remediation.

# V181-004 Verification

Date: 2026-06-27
Executor: Codex
Task: `V181-004` / GitHub issue `#573`

## Commands

```text
bash -n scripts/ai/verify_release_strict.sh scripts/ai/verify_release.sh = PASS
scripts/ai/verify_release_strict.sh v18 = PASS
NTPRO_RELEASE_STRICT_SKIP_BUILD=1 scripts/ai/verify_release.sh v18-strict-provenance = PASS
shasum -a 256 target/release/nautilus = d1762dae5cc5962638fd0c62ce675176cbdcd202d096eee5bf25baabbaad61d6
git status --short = tracked changes present during development validation
git rev-list -n1 ntpro-rust-only-v0.18.0 = 6790688ae46d1b25806f3d1d25146c9b47d43328
cargo --version = cargo 1.95.0 (f2d3ce0bd 2026-03-21)
rustc --version = rustc 1.95.0 (59807616e 2026-04-14)
NTPRO_RELEASE_GATE=1 NTPRO_RELEASE_STRICT_SKIP_BUILD=1 scripts/ai/verify_release_strict.sh v18 = expected FAIL on dirty tracked worktree
NTPRO_RELEASE_STRICT_REQUIRE_HEAD_TAG=1 NTPRO_RELEASE_STRICT_SKIP_BUILD=1 scripts/ai/verify_release_strict.sh v18 = expected FAIL on tag mismatch
corrupted manifest binary.sha256 with NTPRO_RELEASE_STRICT_VERIFY_ONLY=1 = expected FAIL on binary sha256 mismatch
git diff --check = PASS
```

## Result

The v0.18 strict provenance gate records and verifies the release binary path,
binary sha256, binary byte count, source commit, source tree, baseline release
tag, baseline release commit, cargo version, and rustc version. The v0.18.1
release-note draft lists `v18-strict-provenance` as required release evidence.

# V181-003 Verification

Date: 2026-06-27
Executor: Codex
Task: `V181-003` / GitHub issue `#572`

## Commands

```text
scripts/ai/verify_release.sh release-surface-current-guard release-publication-guard = PASS
NTPRO_CURRENT_RELEASE_VERSION=v0.18.0 NTPRO_NEXT_PATCH_VERSION=v0.18.1 NTPRO_NEXT_CAPABILITY_VERSION=v0.19.0 NTPRO_CURRENT_RELEASE_CAPABILITY='Owner-Approved Cancel Recovery Preview' scripts/ai/check_release_surface_current.sh = PASS
NTPRO_CURRENT_RELEASE_VERSION=v0.18.0 NTPRO_CURRENT_RELEASE_NAME='NTPRO Rust-only v0.18.0' scripts/ai/check_github_release_published.sh = PASS
bash -n scripts/ai/check_github_release_published.sh scripts/ai/check_release_surface_current.sh scripts/ai/verify_release.sh = PASS
scripts/ai/verify_fast.sh = PASS
stale v0.17 default / unsupported v0.18 publication guard scan = no matches
git diff --check = PASS
```

## Result

The default release guard path now validates `ntpro-rust-only-v0.18.0` for both
release surface and GitHub Release publication evidence. The v0.18.0 release
remains preview-only: no actual cancel send, no automatic remediation, no
Dashboard cancel controls, and no v0.18.1 release publication.

# V190-010 Verification

Date: 2026-06-28
Executor: Codex
Task: `V190-010` / GitHub issue `#586`

## Commands

```text
bash -n scripts/ai/verify_v19_release_gates.sh scripts/ai/verify_release.sh = PASS
ruby -e 'require "yaml"; YAML.load_file(".github/workflows/release-tag.yml"); YAML.load_file(".github/workflows/rust-cutover-smoke.yml")' = PASS
scripts/ai/verify_v19_release_gates.sh = PASS, checked 27 v190 artifacts and 10 actual-cancel trace cases
scripts/ai/verify_release.sh v19-release-gates = PASS
scripts/ai/verify_v19_actual_cancel_golden_traces.sh = PASS
scripts/ai/run_golden_traces.sh = PASS
python3 scripts/ai/validate_golden_trace_release_scope.py --manifest docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json --trace-glob 'tests/golden/*.jsonl' = PASS, 45 cases, 44 executable replay, 1 schema-only scoped
scripts/ai/verify_release_strict.sh v19 = UNAVAILABLE, current script usage is v18 only
cargo fmt --check -p nautilus-cli = PASS
cargo clippy -p nautilus-cli --all-targets -- -D warnings = PASS
scripts/ai/verify_fast.sh = PASS
git diff --check = PASS
```

## Result

Local validation passed. The gate wires `v19-release-gates`,
`release-v19-release-gates`, release notes, readiness evidence, and JSON/golden
trace checks for owner-approved single-shot actual cancel only. It preserves:
automatic cancel = not included, bulk cancel = not included, Dashboard cancel
button = not included, missing readback = release-blocking, missing approval
provenance = release-blocking, production order submit lifecycle = not
included, and v0.20 enters owner-approved production order lifecycle.

# V181-002 Verification

Date: 2026-06-27
Executor: Codex
Task: `V181-002` / GitHub issue `#571`

## Commands

```text
rg -n "v0\\.17\\.0|v0\\.17\\.1|v0\\.18\\.0|v0\\.18\\.1|v0\\.19\\.0" README.md docs/rust-cutover/versioning.md docs/rust-cutover/release = PASS
stale current/latest v0.17 or future/unpublished v0.18 surface scan = no matches in current release-surface files
NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG=1 scripts/ai/check_release_surface_current.sh = PASS, current_release_version=v0.18.0
scripts/ai/verify_release.sh release-surface-current-guard = PASS, current_release_version=v0.18.0
bash -n scripts/ai/check_release_surface_current.sh = PASS
scripts/ai/verify_fast.sh = PASS
git diff --check = PASS
```

## Result

The public release surface now presents `ntpro-rust-only-v0.18.0` as the
current formal baseline, `v0.18.1` as the release surface and provenance
hardening patch, and `v0.19.0` as the next owner-approved single-shot actual
cancel capability track. The v0.18.0 boundary remains preview-only: no actual
cancel send, no automatic remediation, and no Dashboard cancel controls.

# V181-001 Verification

Date: 2026-06-27
Executor: Codex
Task: `V181-001` / GitHub issue `#570`

## Commands

```text
gh release view ntpro-rust-only-v0.18.0 --json tagName,name,isDraft,isPrerelease,publishedAt,targetCommitish,url = PASS
gh run view 28281346239 --repo atxinbao/NTPRO = PASS, completed success, 50 jobs, 0 failures
stale V180 publication placeholders under docs/rust-cutover/release and verification.md = no matches
published release markers, release commit, hosted gate URL, and preview-only boundary markers = PASS
scripts/ai/verify_release.sh v18-release-gates = PASS
scripts/ai/verify_fast.sh = PASS
git diff --check = PASS
```

## Result

The v0.18.0 release surface is closed out against the live GitHub Release and
hosted release gate. The release remains preview-only: no actual cancel send,
no automatic remediation, and no Dashboard cancel controls.

# V180-011 Verification

Date: 2026-06-26
Executor: Codex
Task: `V180-011` / GitHub issue `#549`

## Commands

```text
test -f docs/rust-cutover/release/v0_18_0_readiness_report.md = PASS
test -f docs/rust-cutover/release/v0_18_0_release_notes.md = PASS
test -f docs/rust-cutover/evidence/V180-011.md = PASS
rg -n "actual cancel send = not included|Dashboard cancel controls = disabled|Actual single-shot cancel remains a v0.19\\+ scope decision|tag = ntpro-rust-only-v0.18.0" docs/rust-cutover/release/v0_18_0_readiness_report.md docs/rust-cutover/release/v0_18_0_release_notes.md = PASS after V181-001 release closeout
scripts/ai/verify_release.sh v18-release-gates = PASS
scripts/ai/verify_fast.sh = PASS
git diff --check = PASS
```

## Result

The V180-011 readiness report and release notes are locally verified. They
account for all V180 tasks and hosted smoke evidence while preserving the
no-send, no-automatic-remediation, no-Dashboard-cancel-control boundary and
stating that actual single-shot cancel remains v0.19+ scope.
