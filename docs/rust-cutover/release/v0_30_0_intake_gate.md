# v0.30.0 Intake Gate

Date: 2026-07-11
Executor: Codex
Task: `V300-000` / GitHub issue `#969`
Milestone: `v0.30.0`

## Start Gate

```text
start_gate_status = satisfied
V291 issues closed = 6/6
V291 milestone = closed
V291 milestone open issues = 0
V291 exact milestone issue set = #963-#968
v0.29.1 release evidence = published
v0.29.1 release closeout evidence = docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md
v0.29.1 hosted release gate = success
v0.29.1 hosted release gate jobs = 90/90 success
v0.29.1 release tag = ntpro-rust-only-v0.29.1
v0.29.1 release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.29.1
v0.29.1 hosted release gate URL = https://github.com/atxinbao/NTPRO/actions/runs/29130876713
v0.29.1 tag object SHA = d3d398530835342dab4aafe355d1c842be0fdd47
v0.29.1 tag SHA = a831d802e4321f50ed6e10481aea35b15a74b01e
v0.29.1 tag is ancestor of origin/main = true
v0.29.1 GitHub Release published at = 2026-07-11T01:07:24Z
v0.29.1 hosted release gate completed at = 2026-07-11T01:06:27Z
v0.29.1 GitHub Release published after hosted gate = true
v0.29.1 GitHub Release body normalized sha256 = 2e11eaa92a91040fdf4e3903b97e58ebcfedeedcc9a1d45a24c56ea2f3a2eef8
v29 publish-after-gate current binding points at v0.29.0 = true
v29 publish-after-gate current binding points at v0.28.0 = false
```

## V291 Closeout

```text
#963 V291-001 = closed
#964 V291-002 = closed
#965 V291-003 = closed
#966 V291-004 = closed
#967 V291-005 = closed
#968 V291-006 = closed
```

## Source Reconstruction

```text
v0.29.1 release manifest = docs/rust-cutover/release/v0_29_1_release_manifest.json
v0.29.1 readiness report = docs/rust-cutover/release/v0_29_1_readiness_report.md
v0.29.1 release notes = docs/rust-cutover/release/v0_29_1_release_notes.md
v0.29.1 release closeout evidence = docs/rust-cutover/release/v0_29_1_release_closeout_evidence.md
V291 task files = docs/rust-cutover/tasks/V291-001.md..V291-006.md
V291 evidence files = docs/rust-cutover/evidence/V291-001.md..V291-006.md
v29.1 release gates = scripts/ai/verify_v29_1_release_gates.sh
v29.1 strict provenance = scripts/ai/verify_v29_1_strict_provenance.sh
v29.1 v30 start gate = scripts/ai/verify_v29_1_v30_start_gate.sh
v30 intake gate = scripts/ai/verify_v30_intake_gate.sh
publication evidence strategy = source_tree_plus_github_remote
local generated publication evidence required in source tree = false
remote reconstruction required = true
```

## V300 Scope

```text
v0.30.0 milestone = open
v0.30.0 milestone issue set = #969-#980
V300 issue count = 12
V300 registered corrective-scope exception count = 0
V300-000 intake gate and v0.29.1 dependency proof = #969
V300-001 backend go-live candidate boundary contract = #970
V300-002 production deployment plan and environment readiness = #971
V300-003 runtime enablement boundary and controlled feature flags = #972
V300-004 operator approval freeze and change-window lifecycle = #973
V300-005 canary execution preflight and no-default-execution gate = #974
V300-006 rollback and disaster recovery execution boundary = #975
V300-007 production config provenance and venue connectivity readiness = #976
V300-008 telemetry SLO gate and incident freeze integration = #977
V300-009 audit retention and evidence export readiness = #978
V300-010 go-no-go runbook and live readiness decision record = #979
V300-011 v30 release gates and v31 production enablement handoff = #980
```

## Boundary Classification

```text
v0.30.0 capability track = backend_production_go_live_candidate_foundation_only
v0.30.0 runtime capability inherited from v0.29.0 = false
v0.30.0 runtime capability inherited from v0.29.1 = false
v0.30.0 trading controls inherited from v0.29.0 = false
v0.30.0 trading controls inherited from v0.29.1 = false
v0.30.0 default production submit = false
v0.30.0 default adapter send = false
v0.30.0 default live exchange request = false
v0.30.0 default automatic remediation = false
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
cancel_order_allowed = false
replace_order_allowed = false
amend_order_allowed = false
flatten_position_allowed = false
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
backend_go_live_claim = false
product_grade_trading_terminal_claim = false
```

## Go Decision

`v0.30.0` may begin only as a gated Backend Production Go-Live Candidate
Foundation track. This intake gate does not authorize default submit, cancel,
retry, replace, amend, flatten, adapter send, live exchange request, retry
scheduling, automatic remediation, Dashboard operation controls, Dashboard
trading controls, Admin Workbench trading controls, Trader Terminal order
tickets, manual operation submit, backend go-live, or product-grade live
trading terminal claims.
