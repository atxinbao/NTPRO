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
