# v0.30.0 Release Closeout Evidence

Date: 2026-07-11
Executor: Codex
Status: PENDING PUBLICATION

This file is the source-controlled closeout target for the public
`ntpro-rust-only-v0.30.0` GitHub Release. It is intentionally present before
tag publication so the v30 release gate can bind the publication evidence path
and reject generated-evidence-only proofs.

## Required Closeout Fields

```text
release tag = ntpro-rust-only-v0.30.0
release name = NTPRO Rust-only v0.30.0
GitHub Release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.30.0
hosted release gate success required = true
publication after hosted gate required = true
strict release body match required = true
source-controlled closeout evidence required = true
generated publication evidence sole proof allowed = false
milestone #26 closeout required = true
v31 handoff remains hard-blocked without v30 release evidence = true
```

## Boundary

```text
backend_go_live_claim = false
actual_backend_production_go_live_allowed = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
automatic_remediation_allowed = false
dashboard_trading_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
```
