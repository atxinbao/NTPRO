# NTPRO v0.3.1 Supervisor Control Hardening Release Closeout

Date: 2026-06-13
Executor: Codex
Milestone: v0.3.1 Local Supervisor Control Console Hardening
Status: final release-accounting source tree; formal publication requires clean hosted PASS, tag push, and GitHub Release on this same commit

## Purpose

This document closes the `v0.3.1` formal release accounting gap for everything
already merged to `main` after `ntpro-rust-only-v0.3.0`.

It has four jobs:

1. record the validated release baseline;
2. classify merged PRs into release-accounting buckets;
3. keep README / readiness / release notes / GitHub Release wording aligned;
4. record the final hosted gate, tag, and GitHub Release publication result.

## Validated Baseline Facts

These facts were re-verified during `v0.3.1` release closeout:

| Item | Value | Evidence source |
| --- | --- | --- |
| Current working branch at audit start | `main` | local git |
| Audit baseline `origin/main` commit at closeout start | `6fd5a68aea938dd7f60f0a443241694b095a0325` | local git / `gh pr list` |
| Refreshed `origin/main` commit before tag preparation | `cbce10b03cfb725a2479b3fc8696f2c646911332` | local git / `gh pr list` |
| Prior formal tag | `ntpro-rust-only-v0.3.0` | local git / GitHub Release |
| Prior formal tag commit | `2822ef8c29771de8ef1b90b96507ac6f1bcefcb3` | local git |
| Latest formal GitHub Release before v0.3.1 publication | `ntpro-rust-only-v0.3.0` | `gh release list` |
| Latest clean hosted gate on baseline `main` during audit | run `27455863969` on `6fd5a68aea938dd7f60f0a443241694b095a0325` | `gh run list` / `gh run view` |
| Additional merged docs-only deltas after closeout start | `#279`, `#280`, `#281` | `gh pr list --state merged` |
| GitHub open PR count at audit start | `0` | `gh pr list --state open` |

## Category Rules

| Category | Meaning |
| --- | --- |
| `A` | Belongs to the v0.3.1 release blocker / hardening / gate / evidence claim. |
| `B` | Docs-only / product-doc-only / planning-only delta merged into the source tree; must be disclosed in release closeout, but does not expand shipped capability. |
| `C` | Source-tree delta included in the tag that affects copy, tests, quality, or planning boundaries; must be disclosed as included, but must **not** be described as a v0.3.1 capability expansion. |

## Merged PR Accounting

All PRs below are already merged to `main` after `ntpro-rust-only-v0.3.0`.

| PR | Title | Category | Included in v0.3.1 tag source tree | Capability expansion | Release treatment |
| --- | --- | --- | --- | --- | --- |
| #258 | Fix release gate linker crash on hosted runners | `A` | Yes | No | Include in hardening and hosted-gate notes. |
| #259 | Localize Dashboard UI copy to Chinese | `C` | Yes | No | Mention as user-facing copy/localization delta only. |
| #260 | Serialize release gate cargo builds | `A` | Yes | No | Include in release-gate hardening notes. |
| #261 | Split release gate into staged jobs | `A` | Yes | No | Include in release-gate hardening notes. |
| #262 | V031-001 Align README with v0.3 release surface | `A` | Yes | No | Include in release-surface hardening notes. |
| #263 | V031-002 Use release binaries in v0.3 smoke checks | `A` | Yes | No | Include in release verification notes. |
| #264 | V031-003 Record hosted release gate evidence | `A` | Yes | No | Include in hosted-evidence closeout notes. |
| #265 | V031-004 Recover stale supervisor registry locks | `A` | Yes | No | Include in supervisor hardening notes. |
| #266 | V031-005 Harden supervisor process identity | `A` | Yes | No | Include in supervisor hardening notes. |
| #267 | V031-006 define supervisor pause/resume semantics | `A` | Yes | No | Include in control-semantics hardening notes. |
| #268 | V031-007 clarify supervisor reconnect contract | `A` | Yes | No | Include in control-semantics hardening notes. |
| #269 | V031-008 negative supervisor control path tests | `A` | Yes | No | Include in release verification notes. |
| #270 | V031-009 ignored test batch closeout | `A` | Yes | No | Include as scoped quality-accounting closure, not new runtime capability. |
| #271 | V031-010 readiness report | `A` | Yes | No | Include as readiness evidence. |
| #272 | Fix dashboard smoke CI fallback | `A` | Yes | No | Include in hosted/dashboard smoke hardening notes. |
| #273 | Split release test and golden trace stages | `A` | Yes | No | Include in release-gate hardening notes. |
| #274 | split release workspace tests | `A` | Yes | No | Include in release-gate hardening notes. |
| #275 | add ntpro console product docs | `B` | Yes | No | Disclose as architecture/product-doc delta only. |
| #276 | scope high precision catalog fixtures | `C` | Yes | No | Disclose as test/quality delta only. |
| #277 | add v0.4 task queue docs | `B` | Yes | No | Disclose as planning-only delta; do not turn into v0.3.1 claim. |
| #278 | refine trader terminal product docs | `B` | Yes | No | Disclose as product-doc delta only; do not turn into trader-terminal release claim. |
| #279 | docs: define v0.4 Binance sandbox boundary | `B` | Yes | No | Disclose as scope-only delta; do not turn into v0.3.1 claim. |
| #280 | docs: classify Binance v0.4 sandbox capability | `B` | Yes | No | Disclose as capability-matrix planning delta only. |
| #281 | docs: define v0.4 EMA RSI strategy contracts | `B` | Yes | No | Disclose as strategy-spec planning delta only. |

## Release-Surface Consistency Rules

For `v0.3.1`, all public release surfaces must say the same thing:

```text
Shipped capability: local sandbox-only Supervisor control console
Release-line description: Local Supervisor Control Console Hardening
Source-tree deltas exist beyond the shipped capability claim
Those deltas do not expand v0.3.1 into v0.4, trader terminal delivery, or production trading
```

Required consistency points:

- `README.md`
- `docs/rust-cutover/release/v0_3_1_supervisor_control_hardening_release_notes.md`
- `docs/rust-cutover/release/v0_3_1_supervisor_control_hardening_readiness_report.md`
- this closeout record
- final GitHub Release body

## Local Verification

Required local verification ran on the release-prep source tree:

| Command | Result | Notes |
| --- | --- | --- |
| `scripts/ai/verify_full.sh fast` | PASS | Fast baseline gate passed. |
| `cargo test -p nautilus-cli dashboard --lib` | PASS | Dashboard lib tests passed: `25 passed; 0 failed`. |
| `cargo test -p nautilus-cli supervisor --lib` | PASS | Supervisor lib tests passed: `29 passed; 0 failed`. |
| `scripts/ai/verify_release.sh release-build-product-surface rust-only-gates v03-supervisor-control-smoke v03-dashboard-smoke` | PASS | Release product surface, Rust-only gates, v0.3 supervisor control smoke, and v0.3 dashboard smoke all passed with release binaries. |
| `git diff --check` | PASS | No whitespace diff errors. |

Key smoke details recorded during local verification:

- v0.3 supervisor control smoke: `status=ok`
- v0.3 dashboard smoke: `status=ok`
- dashboard final states: `sandbox-a=running`, `sandbox-b=stopped`
- release binaries used:
  - `target/release/nautilus`
  - `target/release/ntpro-node`

The later `origin/main` deltas `#279` through `#281` are docs-only and do not
change the runtime or release-binary verification surface covered by these
commands.

## Hosted Release Gate

Hosted release evidence is split into two layers:

- historical baseline context, which shows the gate can pass on `main`;
- final publication evidence, which must be a clean hosted PASS on the exact
  commit used for the `v0.3.1` tag.

Recorded context during closeout:

- baseline `main` already had a clean hosted `Rust Cutover Release Gate` PASS
  on run `27455863969` for commit
  `6fd5a68aea938dd7f60f0a443241694b095a0325`;
- later closeout accounting had to absorb additional docs-only merged deltas
  `#279` through `#281`, moving the publish baseline to
  `cbce10b03cfb725a2479b3fc8696f2c646911332`;
- any branch reruns before the final publish commit are audit context only;
  they do not replace the clean hosted PASS requirement on the final release
  commit.

## Tag And Release Publication

Formal publication is valid only when all of the following point at the same
source tree:

- `main`
- `origin/main`
- tag `ntpro-rust-only-v0.3.1`
- formal GitHub Release body
- hosted `Rust Cutover Release Gate` PASS evidence

## Final Decision

`v0.3.1` may be closed only when all of the following are true:

- local verification passed;
- hosted release gate passed on the release commit;
- `ntpro-rust-only-v0.3.1` tag exists and is pushed;
- formal GitHub Release is published;
- `main` and `origin/main` are aligned;
- merged PR accounting remains disclosed for `#258` through `#281`, including
  docs-only and planning-only source-tree deltas that do not expand the
  shipped capability claim.
