# NTPRO v0.3.1 Supervisor Control Hardening Readiness Report

Date: 2026-06-13
Executor: Codex
Milestone: v0.3.1 Local Supervisor Control Console Hardening
Decision: Final publish-tree readiness PASS; formal publication requires a clean hosted release gate on this exact source tree

## Plain Chinese Summary

v0.3.1 这批修正任务已经把 v0.3.0 发布后的主要问题收口到一个清楚边界：
本地 Supervisor 控制台可以继续作为当前产品面，但它仍然只代表本地 sandbox
控制台能力，不代表生产实盘、真实交易所 reconnect、真实下单或远程多用户控制。

大白话说：这批任务已经把 README、release smoke、GitHub 失败记录、本地进程
状态、pause/resume/reconnect 语义、负向 API 测试和 ignored tests 范围都写清楚
并验证了。本地 `verify_release.sh` 这次完整通过，这份文档就是 `v0.3.1`
最终发布树要带上的 readiness 说明。

正式发布还有一个硬门槛：GitHub hosted `Rust Cutover Release Gate`
必须在同一个发布提交上 clean PASS。这个 hosted PASS 不是可选项，也不能拿旧的
baseline 通过记录替代。因此这份报告的口径是：

```text
发布树 readiness: PASS
本地 release 验证: PASS
正式 tag / GitHub Release: 只有 hosted release gate 在同一提交 clean PASS 后才允许
```

## Scope

Scope source:

- `docs/rust-cutover/scope/v0_3_1_supervisor_control_hardening.md`

Release claim:

```text
Local Supervisor Control Console Hardening
```

Accounting companion documents:

- `docs/rust-cutover/release/v0_3_1_supervisor_control_hardening_release_notes.md`
- `docs/rust-cutover/release/v0_3_1_supervisor_control_hardening_closeout.md`

In scope:

- public README and release wording aligned with v0.3.x;
- release smoke uses release binaries for v0.3 supervisor/dashboard paths;
- hosted release-gate failures are recorded and separated from local evidence;
- local supervisor stale lock and process identity checks are hardened;
- pause/resume are documented as local artifact-level controls;
- reconnect actions remain explicit local sandbox `not_supported` results;
- negative-path API and CLI control requests are tested;
- first ignored-test batch is classified against v0.3.1 scope;
- final report uses strict PASS/FAIL language.

Out of scope:

- production real-exchange live trading;
- real account connectivity;
- real order submission;
- manual order entry;
- production reconnect behavior;
- runtime-level strategy/adapter/execution loop pause or resume;
- remote or distributed dashboard operation;
- Docker or prebuilt binary delivery;
- v0.4 exchange or strategy productization.

## Source-Tree Delta Accounting

This report is the readiness gate for the shipped `v0.3.1` claim. It is not
allowed to silently ignore other PRs already merged into `main`.

The merged-PR accounting for `#258` through `#281` is maintained in:

- `docs/rust-cutover/release/v0_3_1_supervisor_control_hardening_closeout.md`

That accounting separates three things cleanly:

- hardening/gate/evidence work that belongs to the v0.3.1 claim;
- docs/product/planning deltas that are in the source tree;
- source-tree deltas that must be disclosed but must not be turned into new
  shipped capability claims.

## V031 Task Readiness

| Task | Scope | Evidence | PR | Status |
| --- | --- | --- | --- | --- |
| `V031-001` | Public README and release surface cleanup | `docs/rust-cutover/evidence/V031-001.md` | #262 | PASS |
| `V031-002` | Release binary smoke support | `docs/rust-cutover/evidence/V031-002.md` | #263 | PASS |
| `V031-003` | Hosted release gate triage and evidence closeout | `docs/rust-cutover/evidence/V031-003.md` | #264 | PASS as triage and publication requirement cleanup |
| `V031-004` | Registry stale lock recovery | `docs/rust-cutover/evidence/V031-004.md` | #265 | PASS |
| `V031-005` | Process identity hardening | `docs/rust-cutover/evidence/V031-005.md` | #266 | PASS |
| `V031-006` | Pause/resume semantics contract | `docs/rust-cutover/evidence/V031-006.md` | #267 | PASS |
| `V031-007` | Reconnect control contract cleanup | `docs/rust-cutover/evidence/V031-007.md` | #268 | PASS |
| `V031-008` | Negative control API tests | `docs/rust-cutover/evidence/V031-008.md` | #269 | PASS |
| `V031-009` | Ignored tests closure batch 1 | `docs/rust-cutover/evidence/V031-009.md` | #270 | PASS as scope closure |
| `V031-010` | v0.3.1 readiness report | `docs/rust-cutover/evidence/V031-010.md` | current task | PASS as final publish-tree readiness report |

## Local Verification

All required local verification commands were rerun for the release-prep source
tree before publication:

| Command | Result | Summary |
| --- | --- | --- |
| `scripts/ai/verify_full.sh fast` | PASS | Fast release baseline passed. |
| `cargo test -p nautilus-cli dashboard --lib` | PASS | Dashboard lib tests passed: `25 passed; 0 failed`. |
| `cargo test -p nautilus-cli supervisor --lib` | PASS | Supervisor lib tests passed: `29 passed; 0 failed`. |
| `scripts/ai/verify_release.sh release-build-product-surface rust-only-gates v03-supervisor-control-smoke v03-dashboard-smoke` | PASS | Release build product surface, Rust-only gates, v0.3 supervisor control smoke, and v0.3 dashboard smoke all passed with release binaries. |
| `git diff --check` | PASS | No whitespace diff errors. |

Direct release-smoke highlights:

- v0.2 two-node supervisor smoke:
  - result: `v02_two_node_smoke status=ok`
  - root: `/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v02-009.cUzFJx`
- v0.3 supervisor control smoke:
  - result: `v03_supervisor_control_smoke status=ok`
  - root: `/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-control.a6PTfN`
  - release binaries used:
    - `/Users/mac/Documents/NTPRO/target/release/nautilus`
    - `/Users/mac/Documents/NTPRO/target/release/ntpro-node`
- v0.3 dashboard control smoke:
  - result: `v03_dashboard_smoke status=ok`
  - root: `/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-010.yCdjPV`
  - dashboard URL during smoke: `http://127.0.0.1:65276/dashboard`
  - final dashboard states: `sandbox-a=running`, `sandbox-b=stopped`

The additional current-`main` deltas `#279` through `#281` are docs-only and
do not change the runtime verification surface above.

## Hosted Release Gate Requirement

Formal publication must be backed by a clean hosted `Rust Cutover Release Gate`
PASS on this exact publish tree. Historical runs remain useful audit context,
but they do not replace the final PASS requirement on the release commit.

Latest known runs:

| Run | Event | Ref / branch | Commit | Conclusion | URL |
| --- | --- | --- | --- | --- | --- |
| `27423501016` | `workflow_dispatch` | `main` | `5bc497e6e7aa93d615e2d3580c61757de9eb7fbe` | `cancelled` | `https://github.com/atxinbao/NTPRO/actions/runs/27423501016` |
| `27421121134` | `workflow_dispatch` | `main` | `afc805396ad731e93f99252fbf3ca9e81010753a` | `failure` | `https://github.com/atxinbao/NTPRO/actions/runs/27421121134` |
| `27418815065` | `workflow_dispatch` | `main` | `93db1f91b544a19a778d9ded2761c093b949da90` | `cancelled` | `https://github.com/atxinbao/NTPRO/actions/runs/27418815065` |
| `27384342541` | `push` | `ntpro-rust-only-v0.3.0` | `2822ef8c29771de8ef1b90b96507ac6f1bcefcb3` | `failure` | `https://github.com/atxinbao/NTPRO/actions/runs/27384342541` |

Interpretation for release closeout:

- These hosted failures/cancellations were recorded before the final publish
  tree was closed.
- They cannot be used as `v0.3.1` release approval.
- The final publish commit must carry its own clean hosted
  `Rust Cutover Release Gate` PASS.
- Tag creation and GitHub Release publication must wait for that hosted PASS.

## PASS / FAIL Decision

| Decision item | Result | Reason |
| --- | --- | --- |
| V031 hardening queue complete enough to enter hosted release gate | PASS | `V031-001` through `V031-009` are merged with evidence, and the required local release checks passed. |
| Local v0.3.1 readiness | PASS | Local release verification passed with release binaries and scoped supervisor/dashboard smoke. |
| Hosted release gate on the final publish commit | REQUIRED | Publication is authorized only by a clean hosted PASS on this exact source tree. |
| Publish `ntpro-rust-only-v0.3.1` tag or GitHub Release | CONDITIONED ON HOSTED PASS | README, readiness, release notes, and release body must all refer to the same publish commit. |

## Behavior Impact

This report changes only release documentation and evidence.

No runtime behavior changed.

No trading-semantic behavior changed.

No adapter behavior changed.

## Public API Impact

No public API change.

No CLI command shape changed.

## Migration Note Status

No migration note required. This is a release-readiness report, not a user API
or runtime contract change.

## Remaining Risks

- Hosted GitHub release gate has not produced a clean PASS after the V031
  hardening queue.
- Local release build completed, but it is heavy: the release build alone took
  `11m 54s`.
- The ignored-test batch remains scoped out for v0.3.1, not fixed. Runtime,
  adapter, and performance debt stays visible for later milestones.
- `v0.3.1` still does not claim production trading, real exchange reconnect,
  real order submission, or remote/multi-user Dashboard readiness.

## Next Step

1. Run the hosted `Rust Cutover Release Gate` on the final publish commit for
   this source tree.
2. Require a clean hosted PASS.
3. Use that same commit for `ntpro-rust-only-v0.3.1` tag creation and formal
   GitHub Release publication.
