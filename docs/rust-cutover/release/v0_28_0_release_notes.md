# NTPRO Rust-only v0.28.0

Status: RELEASED
Tag: `ntpro-rust-only-v0.28.0`
Release name: `NTPRO Rust-only v0.28.0`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.28.0`
Base release: `ntpro-rust-only-v0.27.1`

v0.28.0 publishes the Backend Closure / Product Operations Runtime
Finalization track. It closes the source-controlled backend closure evidence
line for v0.27.1 dependency proof, backend boundary classification,
identity/permission, persistent audit storage, deployment orchestration,
telemetry/SLO ingestion, Admin Workbench backend state, Trader Terminal backend
API handoff, fail-closed hardening, release gates, and strict provenance.

v0.28.0 publishes the Backend Closure / Product Operations Runtime Finalization track.

Plain Chinese summary: v0.28.0 的范围是 backend closure 和发布治理收口，不是产品级
实盘交易终端。它证明身份/权限、审计存储、部署编排、telemetry/SLO、Admin Workbench 后端状态、
Trader Terminal 后端 API handoff、fail-closed 汇总、release gate 和 strict provenance 都有
可复放证据；它不新增 submit，不改变 adapter，不访问 live exchange，不开放 Dashboard/Admin/Trader
Terminal 交易控件。

## Closed Backend Scope

```text
V280-000 - v0.28.0 intake gate and v0.27.1 dependency proof.
V280-001 - backend closure boundary contract and readiness matrix.
V280-002 - identity and permission runtime closure.
V280-003 - persistent audit storage runtime closure.
V280-004 - deployment upgrade rollback orchestration runtime closure.
V280-005 - telemetry SLO ingestion runtime closure.
V280-006 - Admin Workbench backend state bridge closure.
V280-007 - backend API contract for Trader Terminal handoff.
V280-008 - backend closure fail-closed hardening.
V280-009 - v28 release gates strict provenance and backend closure handoff.
```

## Release Governance

```text
V280 final release scope issue count = 10
V280 final release scope evidence count = 10
V280 exact milestone issue set = #893-#902
V280 registered corrective-scope exception count = 0
registered corrective-scope exceptions required = true
unregistered corrective milestone issues fail closed = true
v28 release gates = required
v28 strict provenance = required
backend closure boundary contract = required
release surface current guard = required
release publication guard = required
release publish after gate = required
hosted release gate success before public GitHub Release = required
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
```

## Boundary

```text
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
network_attempted = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
automatic_operation_action_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
product_grade_trading_terminal_claim = false
```

## Validation Commands

```text
scripts/ai/verify_release.sh v28-intake-gate
scripts/ai/verify_release.sh v28-backend-closure-boundary-contract
scripts/ai/verify_release.sh v28-identity-permission-runtime-closure
scripts/ai/verify_release.sh v28-persistent-audit-storage-runtime-closure
scripts/ai/verify_release.sh v28-deployment-orchestration-runtime-closure
scripts/ai/verify_release.sh v28-telemetry-slo-ingestion-runtime-closure
scripts/ai/verify_release.sh v28-admin-workbench-backend-state-bridge-closure
scripts/ai/verify_release.sh v28-trader-terminal-backend-api-contract-handoff
scripts/ai/verify_release.sh v28-backend-closure-fail-closed-hardening
scripts/ai/verify_release.sh v28-release-gates
scripts/ai/verify_release.sh v28-strict-provenance
scripts/ai/verify_v28_release_gates.sh
scripts/ai/verify_v28_strict_provenance.sh
scripts/ai/check_github_release_published.sh
scripts/ai/publish_ntpro_release_after_gate.sh
```

## Not Included

This release does not include frontend product completion, product-grade live
trading terminal readiness, default submit, production order mutation, adapter
send, live exchange request, retry scheduler, automatic remediation,
strategy-driven production execution, shared approval consumption, real-funds
proof in CI, or Dashboard/Admin/Trader Terminal order, approval, cancel, retry,
submit, replace, amend, flatten, remediation, or order-ticket controls.

The next patch track is `v0.28.1`. The next capability track is `v0.29.0`.
Neither track inherits production submit, mutation, adapter send, live exchange
request, retry scheduler, automatic remediation, Dashboard/Admin trading
controls, Trader Terminal order tickets, or product-grade live trading terminal
claims from v0.28.0.
