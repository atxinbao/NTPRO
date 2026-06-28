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
