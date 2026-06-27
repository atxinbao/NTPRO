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
