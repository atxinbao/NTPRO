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
