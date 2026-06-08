# NTPRO v0.3 Dashboard MVP Readiness Report

Date: 2026-06-08
Executor: Codex
Milestone: v0.3 Dashboard MVP - Local System Status Cockpit
Decision: PASS

## Plain Chinese Summary

v0.3 Dashboard MVP 可以进入 review。大白话说：现在本地可以启动一个 Dashboard 页面，看到两个 sandbox 节点的状态，看到 Overview、Nodes、Controls、Data Sources、Execution Gateways、Risk Engine、Runtime Modules、Logs / Metrics、Alerts 和 Gaps；页面上的 start/stop 按钮也能通过本地 supervisor 控制 sandbox 节点。

这不是生产交易终端，也不是正式 release 授权。本报告只证明 Dashboard MVP 的本地系统状态看板范围通过。

## Scope Decision

Scope source: `docs/rust-cutover/scope/v0_3_dashboard_mvp.md`.

The v0.3 scope is:

```text
Dashboard MVP - Local System Status Cockpit
```

The product boundary remains:

```text
SupervisorRegistry
  + NodeStatus
  + NodeMetrics
  + local log artifacts
  -> DashboardSnapshot
  -> local HTTP API
  -> static Dashboard UI
```

Dashboard controls are limited to local supervisor lifecycle actions:

```text
start
stop
```

Unsupported actions are visible only as disabled/not-supported controls.

## V03 Task Readiness

| Task | Scope | Evidence | PR | Status |
| --- | --- | --- | --- | --- |
| V03-001 | Scope decision / Dashboard MVP | `docs/rust-cutover/evidence/V03-001.md` | #214 | PASS |
| V03-002 | `DashboardSnapshot` DTO | `docs/rust-cutover/evidence/V03-002.md` | #215 | PASS |
| V03-003 | Status detail DTOs | `docs/rust-cutover/evidence/V03-003.md` | #216 | PASS |
| V03-004 | Supervisor artifacts to snapshot aggregator | `docs/rust-cutover/evidence/V03-004.md` | #222 | PASS |
| V03-005 | Local dashboard HTTP server | `docs/rust-cutover/evidence/V03-005.md` | #223 | PASS |
| V03-006 | Overview and Nodes UI | `docs/rust-cutover/evidence/V03-006.md` | #224 | PASS |
| V03-007 | Data source / execution / risk panels | `docs/rust-cutover/evidence/V03-007.md` | #225 | PASS |
| V03-008 | Runtime modules diagnostic panel | `docs/rust-cutover/evidence/V03-008.md` | #226 | PASS |
| V03-009 | Dashboard start/stop controls | `docs/rust-cutover/evidence/V03-009.md` | #228 | PASS |
| V03-010 | Dashboard smoke and readiness report | `docs/rust-cutover/evidence/V03-010.md` | #229 | PASS |

Agentflow state was reconciled during V03-010 so V03-003 through V03-009 now record their merged PRs and `DONE` status.

## Dashboard Smoke Evidence

Command:

```bash
scripts/ai/v03_dashboard_smoke.sh
```

Result: PASS.

Smoke output summary:

```text
v03_dashboard_smoke status=ok
root=/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-010.ICobrE
registry=/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-010.ICobrE/supervisor/registry.json
dashboard_url=http://127.0.0.1:51064/dashboard
artifacts=/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-010.ICobrE/browser
nodes=sandbox-a,sandbox-b
final_dashboard_states={'sandbox-a': 'running', 'sandbox-b': 'stopped'}
```

Browser evidence captured:

- desktop snapshot: `/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-010.ICobrE/browser/desktop-snapshot.txt`
- desktop screenshot: `/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-010.ICobrE/browser/desktop.png`
- desktop assertions: `/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-010.ICobrE/browser/desktop-assertions.txt`
- mobile snapshot: `/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-010.ICobrE/browser/mobile-snapshot.txt`
- mobile screenshot: `/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-010.ICobrE/browser/mobile.png`
- mobile layout assertions: `/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-010.ICobrE/browser/mobile-layout.txt`
- final snapshot JSON: `/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-010.ICobrE/browser/final-snapshot.json`

The smoke verified:

- Dashboard page opens locally.
- Two sandbox nodes are visible.
- Overview and Nodes render real supervisor state.
- Controls render start/stop and disabled unsupported actions.
- Start button starts `sandbox-a` through local supervisor.
- Stop button stops `sandbox-b` through local supervisor.
- Data Sources, Execution Gateways, Risk Engine, Runtime Modules, Logs / Metrics, Alerts, and Gaps render from `DashboardSnapshot`.
- Desktop and mobile layouts pass automated visibility/overflow checks.
- No production venue connection or real order submission is reported.

## Required Validation

| Command | Result | Summary |
| --- | --- | --- |
| `cargo fmt --check` | PASS | Formatting check passed. |
| `cargo check -p nautilus-cli` | PASS | CLI crate compiled. |
| `cargo test -p nautilus-cli dashboard --lib` | PASS | 22 passed, 0 failed. |
| `cargo test -p nautilus-cli supervisor --lib` | PASS | 16 passed, 0 failed. |
| `scripts/ai/v02_two_node_supervisor_smoke.sh` | PASS | Two sandbox nodes registered, started, inspected, and stopped. |
| `scripts/ai/v03_dashboard_smoke.sh` | PASS | Dashboard browser smoke passed with start/stop controls. |
| `scripts/ai/check_rust_only_runtime.sh` | PASS | Rust-only product surface gate passed. |
| `scripts/ai/verify_fast.sh` | PASS | Fast smoke passed; this is not a full release gate. |
| `git diff --check` | PASS | No whitespace diff errors. |

## Out Of Scope

The following are not approved by this report:

- release tag creation;
- GitHub Release publication;
- production real-exchange live trading;
- manual order entry;
- order modification;
- order cancellation;
- strategy parameter hot reload;
- production reconnect controls;
- remote or distributed dashboard operation;
- multi-user permissions;
- full release verification beyond the required v0.3 Dashboard MVP checks.

## Behavior Impact

The dashboard now exposes a Logs / Metrics panel so users can inspect the local supervisor log and metric artifacts from the same page. Mobile layout constraints were tightened so the required panels do not produce obvious page-level horizontal overflow at a 390px viewport.

Start/stop behavior remains local-only and sandbox-only. The dashboard still does not mutate trading internals directly; it requests lifecycle actions through the local supervisor path.

## Public API Impact

No Rust library API was added. The public CLI/UI surface gains a fuller local dashboard page and a repeatable smoke script:

```text
scripts/ai/v03_dashboard_smoke.sh
```

## Migration Note

No migration note is required. This is a Dashboard MVP readiness and public UI cleanup task, not a breaking CLI or Rust API change.

## Remaining Risks

- This report does not replace `scripts/ai/verify_release.sh`.
- Browser evidence is local and temporary; the readiness decision depends on the recorded command output and artifact paths from this run.
- Unsupported runtime internals still appear explicitly as `not_supported` or `unknown`; that is expected for v0.3 and must not be interpreted as production readiness.

## Final Decision

PASS.

NTPRO v0.3 Dashboard MVP is ready for review under the local Dashboard MVP scope. Tag creation or GitHub Release publication still requires separate explicit user approval.
