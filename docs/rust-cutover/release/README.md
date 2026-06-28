# Release Evidence

Final release checks, benchmark summaries, and audit reports live here.

- `BACKTEST_LIVE_GATE_EVIDENCE.md` - RBTL-010 backtest/live gate evidence and
  remaining blockers.
- `binary_install_path.md` - NBIN-001 source-build, local `cargo install`,
  binary naming, platform scope, artifact strategy, and Docker deferral
  decision.
- `v0_2_local_multi_node_readiness_report.md` - V02-010 final PASS/FAIL
  readiness report for the local multi-node runtime foundation.
- `v0_4_1_binance_sandbox_release_surface_hardening_readiness_report.md` -
  V041-005 final PASS/FAIL readiness report for the v0.4.1 Binance sandbox
  release-surface hardening patch.
- `v0_4_1_release_notes.md` - V041-005 release notes for the
  `ntpro-rust-only-v0.4.1` GitHub Release.
- `v0_5_0_workflow_artifacts_readiness_report.md` - V05-011 final PASS/FAIL
  readiness report for the v0.5 local Binance sandbox workflow artifact scope.
- `v0_6_0_binance_testnet_dry_run_readiness_report.md` - V06-012 final
  PASS/FAIL readiness report for the v0.6 offline Binance testnet dry-run
  foundation.
- `v0_6_0_release_notes.md` - release notes for the
  `ntpro-rust-only-v0.6.0` GitHub Release. This release absorbs the completed
  `v0.5.0` workflow-artifact foundation and does not publish a separate
  `v0.5.0` tag/release.
- `v0_6_1_offline_hardening_readiness_report.md` - V061-008 final PASS/FAIL
  readiness report for the v0.6.1 offline hardening track.
- `v0_6_1_release_notes.md` - release notes for the v0.6.1 hardening scope.
  This is not a real Binance testnet network release and does not claim
  production trading.
- `v0_7_0_readonly_testnet_boundary.md` - V070-000 boundary, threat model, gate
  split, and artifact schema for the v0.7 real Binance testnet read-only
  connectivity proof. This document is a high-risk design gate and does not
  implement networking.
- `v0_7_0_readonly_testnet_readiness_report.md` - V070-007 final readiness
  report for the scoped v0.7 real Binance testnet read-only connectivity proof.
- `v0_7_0_release_notes.md` - release notes for the scoped v0.7 read-only
  Binance testnet connectivity proof. It explicitly excludes testnet
  order submission, production Binance connectivity, real funds, production
  trading, and production parity.
- `v0_7_1_release_gate_hardening_readiness_report.md` - released readiness
  report for the v0.7.1 release-gate, artifact-contract, and public-boundary
  hardening patch.
- `v0_7_1_release_notes.md` - released notes for the v0.7.1 hardening scope.
  This is not a new trading capability release and does not claim order
  submission, production connectivity, real funds, or production trading.
- `v0_7_2_readiness_report.md` - released PASS readiness report for the
  v0.7.2 wording/evidence patch. The formal tag
  `ntpro-rust-only-v0.7.2` and GitHub Release are complete.
- `v0_7_2_release_notes.md` - released notes for the v0.7.2 wording/evidence
  patch. It does not expand the v0.7 capability claim and does not include
  order submission, real funds, or production trading.
- `v0_8_0_authenticated_readonly_boundary.md` - v0.8 authenticated Binance
  testnet read-only proof boundary, endpoint allowlist, secret handling, and
  Dashboard artifact rules.
- `v0_8_0_authenticated_readonly_readiness_report.md` - released readiness
  report for the scoped v0.8 authenticated Binance testnet read-only proof.
- `v0_8_0_release_notes.md` - released notes for the scoped authenticated
  Binance testnet read-only proof. It does not include order submission,
  production Binance connectivity, real funds, or production trading.
- `v0_8_1_readiness_report.md` - readiness report for the v0.8.1
  safety/closure patch. It does not create a tag and does not publish a GitHub
  Release.
- `v0_8_1_release_notes.md` - release-note draft for a possible
  owner-approved v0.8.1 safety/closure release. It does not add order
  submission, account mutation, production Binance connectivity, real funds, or
  production trading.
- `v0_9_0_strategy_runtime_boundary.md` - V090-000 boundary document for the
  v0.9.0 Strategy Runtime Foundation milestone. It explicitly defers Binance
  testnet order proof to v0.10.0 and excludes order submission, real funds,
  production trading, and Dashboard order controls.
- `v0_9_0_signal_artifact_contract.md` - V090-005 signal JSONL contract for
  local Strategy Runtime signals. It defines required fields and keeps signals
  separate from order intents, exchange orders, and production trading claims.
- `v0_9_0_order_intent_artifact_contract.md` - V090-006 order intent JSONL
  contract for local Strategy Runtime shadow order intents. It requires
  `submission_allowed=false` and keeps intents out of execution adapters and
  exchange APIs.
- `v0_9_0_risk_decision_gate.md` - V090-007 shadow risk decision gate for local
  order intents. It requires `decision=rejected` and `actual_submission=false`
  and keeps the v0.9 flow stopped before execution adapters or exchange APIs.
- `v0_9_0_strategy_session_audit_log.md` - V090-008 Strategy Session audit log
  and summary contract. It records lifecycle/risk decision events and
  `summary.json` counts while preserving the no-execution v0.9 boundary.
- `v0_9_0_ntpro_node_strategy_session_host.md` - V090-009 `ntpro-node`
  Strategy Session host contract. It runs fixture strategy sessions and writes
  local artifacts while preserving the no-order, no-production v0.9 boundary.
- `v0_9_0_supervisor_strategy_session_status.md` - V090-010 supervisor
  Strategy Session read-only status surface. It exposes strategy state, market
  state, risk state, counts, and artifact paths without adding trading
  controls.
- `v0_9_0_dashboard_strategy_runtime_readonly.md` - V090-011 Dashboard
  Strategy Runtime read-only surface. It displays local session, signal,
  order-intent, risk-decision, rejection, and artifact-path fields without
  adding Dashboard order controls.
- `v0_9_0_strategy_runtime_smoke_gates.md` - V090-012 PR and release gate
  wiring for local Strategy Runtime smoke and shadow-mode no-order verification.
- `v0_9_0_strategy_runtime_readiness_report.md` - released readiness report for
  the v0.9.0 Strategy Runtime Foundation milestone.
- `v0_9_0_release_notes.md` - released notes for the formal
  `ntpro-rust-only-v0.9.0` GitHub Release.
- `v0_9_1_readiness_report.md` - release-readiness closure report for the
  v0.9.1 Strategy Runtime semantics and audit hardening patch. It records the
  completed V091 queue and waits for owner release decision; it does not create
  a tag or publish a GitHub Release.
- `v0_9_1_release_notes.md` - release-note material for a possible
  owner-approved v0.9.1 Strategy Runtime hardening release. It keeps Binance
  testnet order proof deferred to v0.10.0 and remains unpublished until owner
  approval.
- `v0_10_0_order_boundary.md` - released boundary document for the v0.10.0
  Binance spot sandbox order proof milestone. It defines the Spot Test Network
  and Spot Demo Mode sandbox allowlist, owner/manual-gated order-test, tiny
  submit, cancel, reconciliation, redaction, and read-only Dashboard artifact
  boundary while preserving no production Binance, no real funds, no production
  trading, and no Dashboard order controls.
- `v0_10_0_execution_config_contract.md` - V100-001 execution config contract
  for the disabled-by-default `[testnet_order]` section, including allowlisted
  spot sandbox URL, one-symbol alignment, string decimal
  price/quantity/notional fields, owner/manual gates, and forbidden
  production/Dashboard order controls.
- `v0_10_0_order_gate.md` - V100-002 multi-layer order gate contract for
  fail-closed CLI flags and env vars. It adds local gate readiness only and
  keeps network attempts, real orders, and Dashboard order controls disabled.
- `v0_10_0_risk_preflight.md` - V100-003 offline order risk preflight contract
  for local session, market, account, kill switch, symbol allowlist, limit,
  clock skew, and endpoint checks before future order request construction.
- `v0_10_0_signed_order_request.md` - V100-004 signed Binance spot sandbox
  order request preview contract. It only builds redacted request metadata for
  allowlisted order endpoints and keeps networking, order submission, secrets,
  signatures, signed queries, and signed URLs out of artifacts.
- `v0_10_0_order_test_preflight.md` - V100-005 offline
  `POST /api/v3/order/test` preflight contract. It records redacted request
  shape readiness while explicitly keeping Binance acceptance, matching-engine
  submission, networking, and real orders out of the offline proof.
- `v0_10_0_execution_artifact_contract.md` - V100-007 execution artifact
  contract for request, order-test, submit ack, cancel ack, lifecycle, and
  reconciliation evidence. It keeps V100-006 manual submit/cancel proof
  separate and records offline counters as zero.
- `v0_10_0_reconciliation_fixture.md` - V100-008 offline reconciliation and
  orphan-order fixture contract. It records submit-without-local-ack,
  cancel-timeout, local-open/exchange-filled, and restart-unfinished-order
  states as risk-halted and blocks new orders without network or order
  submission.
- `v0_10_0_dashboard_readonly_order_proof.md` - V100-009 read-only Dashboard
  order proof display. It surfaces risk preflight, order-test, submit/cancel
  ack, terminal lifecycle, reconciliation, and order counters without adding
  Dashboard order or cancel controls.
- `v0_10_0_release_gates.md` - V100-010 offline and manual release gate wiring.
  It adds default offline fail-closed/schema/redaction gates and a separate
  manual order-proof artifact validator without completing V100-006 or running
  real Binance testnet submit/cancel in CI.
- `v0_10_0_tiny_submit_cancel.md` - V100-006 owner-gated manual Binance spot
  sandbox tiny submit-and-cancel runner and proof record. The published
  v0.10.0 proof used Binance Spot Demo Mode `https://demo-api.binance.com`.
  The default command remains closed and does not open network connections or
  submit orders; any future manual proof still requires owner gates, sandbox
  credentials, and validated redacted artifacts.
- `v0_10_0_readiness_report.md` - released readiness report for the v0.10.0
  Binance spot sandbox order proof milestone. It records the completed V100
  queue and owner-confirmed Spot Demo Mode submit/cancel proof while preserving
  the no-production, no-real-funds, no-Dashboard-order-controls boundary.
- `v0_10_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.10.0` GitHub Release. It preserves the Binance spot
  sandbox/manual-gated boundary and does not claim production trading, real
  funds, or Dashboard order controls.
- `v0_11_0_readiness_report.md` - released readiness report for the v0.11.0
  Production Read-Only Contract + Offline Shadow Portfolio milestone. It
  records the completed V110 queue, formal tag `ntpro-rust-only-v0.11.0`, and
  production read-only contract / offline shadow boundary.
- `v0_11_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.11.0` GitHub Release. It preserves the production
  read-only contract / offline shadow boundary and does not claim successful
  online production reads, production order mutation, real funds, production
  trading, or Dashboard order controls.
- `v0_11_1_readiness_report.md` - readiness material for an owner-approved
  v0.11.1 production read-only contract hardening patch. It records the V111
  queue, keeps `ntpro-rust-only-v0.11.0` as the current published release until
  owner publication, and does not create a tag or publish a GitHub Release.
- `v0_11_1_release_notes.md` - release-note material for a possible
  `ntpro-rust-only-v0.11.1` hardening patch. It does not expand the v0.11
  capability claim and does not include successful online production reads,
  production order mutation, real funds, production trading, or Dashboard order
  controls.
- `v0_12_0_boundary.md` - design boundary for the v0.12.0 Production Online
  Read-Only + Persistent Shadow candidate. It moves Guarded Live Alpha to the
  earliest possible v0.13.0 track and keeps v0.12 production mutation out of
  scope.
- `v0_12_0_response_shape.md` - redacted account snapshot response-shape
  evidence contract for v0.12 production account read-only proof.
- `v0_12_0_shadow_portfolio_runtime.md` - local shadow portfolio runtime
  contract for v0.12 redacted read-only inputs and shadow intents.
- `v0_12_0_persistent_shadow_strategy_session.md` - local persistent shadow
  strategy session event contract for v0.12 artifacts.
- `v0_12_0_production_readonly_reconciliation.md` - local read-only
  reconciliation classification contract for v0.12 shadow evidence.
- `v0_12_0_dashboard_production_shadow_readonly_panel.md` - Dashboard
  read-only production shadow panel contract for v0.12 artifacts.
- `v0_12_0_release_gates.md` - v0.12 offline release gate and manual-online
  fail-closed preflight documentation.
- `v0_12_0_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.12.0` Production Online Read-Only + Persistent Shadow
  release.
- `v0_12_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.12.0` GitHub Release. It does not claim production order
  submission, order mutation, order-state reads, real funds, production
  trading, automatic remediation, or Dashboard order controls.
- `v0_12_1_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.12.1` Production Read-Only Evidence & Release Surface
  Hardening patch.
- `v0_12_1_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.12.1` GitHub Release. It does not expand the v0.12
  capability claim and does not include production order submission, order
  mutation, order-state reads, listenKey lifecycle, signed WebSocket user stream
  runtime, real funds, production trading, or Dashboard order controls.
- `v0_13_0_scope_decision.md` - scope decision for the v0.13.0 Guarded Live
  Alpha Preflight line. It limits v0.13.0 to preflight evidence only and does
  not authorize production order submission, order mutation, real funds,
  production trading, or Dashboard order controls.
- `v0_13_0_shadow_session_preflight.md` - V130-002 local shadow preflight
  session contract. It adds heartbeat, stop-file, and stale-data evidence while
  preserving the no-production-mutation and no-Dashboard-order-controls
  boundary.
- `v0_13_0_online_readonly_proof_pack.md` - V130-003 owner-run production
  online read-only proof-pack contract. Default runs remain offline and
  fail-closed; optional owner-run artifacts are redacted and do not expand
  production trading, order mutation, order-state read, listenKey, or Dashboard
  control capability.
- `v0_13_0_kill_switch_approval_artifact.md` - V130-004 local kill-switch
  dry-run/manual-approval artifact contract. It records preflight approval
  evidence while preserving the no-network, no-production-mutation, and
  no-Dashboard-order-controls boundary.
- `v0_13_0_dashboard_control_boundary.md` - V130-005 trader/ops Dashboard
  control boundary. It keeps trader-visible surfaces read-only, limits ops
  controls to local supervisor lifecycle actions, and preserves disabled
  production order controls, credential entry, listenKey lifecycle, production
  reconnect, real funds, and production trading.
- `v0_13_0_decimal_amount_boundary.md` - V130-006 Decimal/string-only amount
  boundary for future live-alpha risk/execution preflight fields. It rejects
  scientific notation, preserves `rust_decimal` aggregation evidence, and keeps
  live-alpha money math, production mutation, real funds, and production
  trading out of v0.13.
- `v0_13_1_money_price_quantity_contract.md` - V131-005 Money/Price/Quantity
  contract draft for future live-alpha preflight. It documents required
  precision, tick size, step size, min/max notional, fee, slippage, and
  rounding-mode inputs while explicitly keeping v0.13.1 out of
  risk/execution-grade money math, production mutation, real funds, and
  production trading.
- `v0_13_1_readiness_report.md` - V131-007 readiness report for the v0.13.1
  Guarded Live Alpha Preflight hardening candidate. It accounts for V131-001
  through V131-006 and keeps tag creation and GitHub Release publication as an
  owner release decision.
- `v0_13_1_release_notes.md` - release-note material for the potential
  `ntpro-rust-only-v0.13.1` GitHub Release. It is hardening-only and does not
  include production order submission, order mutation, order-state reads,
  listenKey lifecycle, real funds, production trading, risk/execution-grade
  live-alpha money math, or Dashboard order controls.
- `v0_14_0_order_state_readonly_boundary.md` - V140-000 production
  order-state read-only boundary. It allows only future owner-gated production
  `GET /api/v3/openOrders` and `GET /api/v3/order` proof scope while keeping
  default execution offline/fail-closed and excluding production order
  submission, mutation, listenKey lifecycle, real funds, production trading,
  automatic remediation, and Dashboard order controls.
- `v0_14_0_readiness_report.md` - released readiness report for the formal
  v0.14.0 Production Order-State Read-Only + Live Alpha Dry-Run release. It
  records the formal tag `ntpro-rust-only-v0.14.0`, accounts for V140-000
  through V140-008, and preserves the no-production-mutation boundary.
- `v0_14_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.14.0` GitHub Release. It preserves owner-gated read-only
  order-state proof and live-alpha dry-run scope and does not include
  production order submission, production mutation, listenKey lifecycle, real
  funds, production trading, or Dashboard order controls.
- `v0_14_1_order_state_owner_evidence.md` - V141-001 owner-run order-state
  read-only evidence contract. It validates redacted `GET /api/v3/openOrders`
  evidence, optionally validates `GET /api/v3/order` evidence, and keeps
  production order submission, mutation, listenKey lifecycle, real funds,
  production trading, automatic remediation, and Dashboard order controls out
  of scope.
- `v0_14_1_readiness_report.md` - V141-007 readiness report for the
  v0.14.1 Order-State Read-Only Evidence Hardening candidate. It accounts for
  V141-001 through V141-006 and keeps tag creation and GitHub Release
  publication as an owner release decision.
- `v0_14_1_release_notes.md` - release-note material for the potential
  `ntpro-rust-only-v0.14.1` GitHub Release. It is hardening-only and does not
  include production order submission, production order mutation,
  cancel/replace/amend/retry/correction, listenKey lifecycle, real funds,
  production trading, or Dashboard order controls.
- `v0_15_0_mutation_scope_decision.md` - V150-000 production mutation scope
  decision. It defines v0.15.0 as Guarded Live Alpha Mutation Scope +
  Execution Dry-Run Harness only: one venue/account label/symbol/order type,
  tiny notional, manual owner approval, kill switch active by default, no
  autonomous strategy execution, no request sending, no production mutation,
  no listenKey lifecycle, no real funds, no production trading, and no
  Dashboard order controls.
- `v0_15_0_manual_approval_lifecycle.md` - V150-005 one-time manual approval
  lifecycle for production live-alpha request preview. It binds approval to
  run, strategy, symbol, notional, and expiry, but still allows only local
  dry-run request preview artifacts and no production mutation.
- `v0_15_0_mutation_dry_run_golden_traces.md` - V150-006 executable golden
  traces for live-alpha mutation dry-run rejection and preflight states. It
  proves every scoped trace keeps production order submission, production
  mutation, network access, and production execution adapter calls disabled.
- `v0_15_0_incident_rollback_artifact.md` - V150-007 manual incident,
  rollback, and emergency-stop artifact contract. It defines evidence-only
  artifacts and rejects automatic remediation, production cancel/correction/
  retry, real exchange mutation, networking, and production adapter calls.
- `v0_15_0_dashboard_mutation_preflight_panel.md` - V150-008 read-only
  Dashboard panel for v0.15 live-alpha mutation preflight artifacts. It shows
  owner approval, redacted request preview, dry-run execution, runtime gate,
  order-state, and boundary counters without adding order/cancel/replace/
  amend/retry/reconnect controls.
- `v0_15_0_readiness_report.md` - released readiness report for the formal
  v0.15.0 Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness
  release. It records the formal tag `ntpro-rust-only-v0.15.0`, accounts for
  V150-000 through V150-010, and preserves the no-production-mutation
  boundary.
- `v0_15_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.15.0` GitHub Release. It explicitly excludes production
  request sending, production order submission, production order mutation,
  cancel/replace/amend/retry/correction, listenKey lifecycle, real funds,
  production trading, automatic remediation, and Dashboard order controls.
- `v0_16_0_production_mutation_scope.md` - V160-001 scope contract for the
  `Minimum Owner-Approved Production Order Mutation Candidate`. It permits only
  a future one-owner-approved tiny `LIMIT` `GTC` production order candidate and
  explicitly excludes strategy-driven production execution, multiple orders,
  `MARKET` orders, cancel/replace/amend/retry/correction/flatten, automatic
  remediation, listenKey lifecycle, multi-venue/multi-account execution, and
  Dashboard order controls.
- `v0_16_0_response_redaction.md` - V160-006 production mutation response
  redaction contract. It requires redacted response artifacts and keeps raw
  exchange payloads, headers, signatures, signed URLs, credentials, and account
  balances out of persisted evidence.
- `v0_16_0_audit_trail.md` - V160-009 production mutation audit-trail contract.
  It records owner approval, request, guarded-send, readback, terminal outcome,
  and boundary counters while preserving no retry/remediation and no Dashboard
  order controls.
- `v0_16_0_failure_semantics.md` - V160-010 failure-mode and no-retry
  semantics for the production mutation candidate. It treats timeout, HTTP
  failure, malformed response, readback mismatch, and kill-switch transition as
  terminal and blocks automatic retry or remediation.
- `v0_16_0_readiness_report.md` - released readiness report for the formal
  v0.16.0 Minimum Owner-Approved Production Order Mutation Candidate release.
  It records the formal tag `ntpro-rust-only-v0.16.0`, accounts for V160-001
  through V160-013, and preserves the single-owner-approved `LIMIT` `GTC`
  candidate boundary.
- `v0_16_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.16.0` GitHub Release. It limits the claim to one tiny
  owner-approved `LIMIT` `GTC` production order candidate and does not include
  strategy live trading, multi-order execution, cancel/replace/amend/retry,
  listenKey lifecycle, real-funds proof in CI, or Dashboard order controls.
- `v0_17_0_readiness_report.md` - released readiness report for the formal
  v0.17.0 Production Reconciliation And Orphan Recovery Evidence release. It
  records the formal tag `ntpro-rust-only-v0.17.0`, accounts for V170-000
  through V170-009, and preserves the local/offline reconciliation and
  orphan-risk evidence boundary.
- `v0_17_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.17.0` GitHub Release. It limits the claim to local
  ledger, redacted readback mapping, reconciliation classification, orphan risk,
  restart recovery, read-only Dashboard evidence, and incident semantics; it
  does not include network readback execution, new production order submission,
  production order mutation, actual cancel send, automatic remediation,
  real-funds proof in CI, or Dashboard order/cancel controls.
- `v0_18_0_cancel_recovery_artifact_contracts.md` - V180-002 cancel recovery
  preview artifact contracts for the formal v0.18.0 Owner-Approved Cancel
  Recovery Preview line. It defines preview-only cancel intent, approval,
  request, response, and post-cancel readback evidence while preserving no
  actual cancel send.
- `v0_18_0_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.18.0` Owner-Approved Cancel Recovery Preview release. It
  records the formal tag, GitHub Release URL, release commit, hosted release
  gate, V180 task accounting, and preview-only no-actual-cancel boundary.
- `v0_18_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.18.0` GitHub Release. It limits the claim to cancel
  recovery preview evidence, owner approval lifecycle, failure/rollback
  evidence, release gates, and Dashboard diagnostics; it does not include
  actual cancel send, automatic cancel, remediation, or Dashboard order/cancel
  controls.
- `v0_18_1_release_notes.md` - draft notes for the v0.18.1 Release Surface &
  Provenance Hardening patch. It lists `v18-strict-provenance` and
  `scripts/ai/verify_release_strict.sh v18` as required release evidence and
  preserves the v0.18.0 preview-only boundary.
- `v0_18_1_release_manifest.json` - machine-readable v0.18.1 release manifest
  for release surface/provenance hardening. It records the v0.18.0 baseline,
  planned/actual patch tag fields, release gates, source/binary provenance
  handoff to `verify_release_strict.sh v18`, and no-actual-cancel boundary
  flags.
- `v0_19_0_actual_cancel_safety_contract.md` - V190-002 safety contract for
  the v0.19 owner-approved single-shot actual cancel line. It defines the
  one approval / one order / one venue / one execution attempt boundary,
  required artifacts, fail-closed reasons, and forbidden automatic, bulk,
  multi-account, multi-strategy, multi-venue, retry, and Dashboard operation
  expansions without implementing a cancel executor.
- `v0_19_0_owner_approval_execution_lifecycle.md` - V190-003 owner approval
  execution lifecycle for the v0.19 actual cancel line. It adds the local
  `production-mutation-actual-cancel-owner-approval-lifecycle` evidence command,
  binds approval to the V190-002 safety contract, release manifest, risk gate,
  order lineage, symbol, account label, venue, owner identity, timestamp, and
  reason, and fail-closes reused, expired, rejected, audited, missing, or
  mismatched approvals without sending a cancel request.
- `v0_19_0_cancel_executor_adapter_boundary.md` - V190-005 cancel executor
  adapter boundary for the v0.19 actual cancel line. It adds the local
  `production-mutation-actual-cancel-executor-adapter-boundary` evidence
  command, binds a V190-003 owner approval lifecycle to an adapter capability
  declaration, records request/response/readback/audit contracts and adapter
  failure taxonomy, and fail-closes unsupported venue, unsupported order-id
  type, missing capability, bulk, retry, automatic cancel, or Dashboard
  execution requests without sending a cancel request.
- `v0_19_0_single_shot_cancel_command.md` - V190-004 single-shot actual cancel
  command for the v0.19 line. It adds the manual-online
  `production-mutation-actual-cancel-single-shot` command, validates owner
  approval, risk gate, release provenance, adapter boundary/capability, and
  owner-supplied order identity, records one attempted `DELETE /api/v3/order`
  through a redacted audit artifact, and blocks missing, reused, mismatched,
  unsupported, automatic, bulk, retry, or Dashboard paths before any send.
- `v0_19_0_post_cancel_readback_reconciliation.md` - V190-006 post-cancel
  readback reconciliation for the v0.19 actual cancel line. It adds the
  `production-mutation-actual-cancel-readback-reconciliation` evidence command,
  requires a recorded V190-004 actual cancel attempt plus redacted readback
  metadata, classifies cancel confirmed, already cancelled, filled before
  cancel, unknown, timeout, and inconsistent outcomes, and keeps degraded
  readback states explicit for Dashboard read-only audit consumption without
  retry, remediation, second cancel, or network readback execution.
- `v0_19_0_actual_cancel_failure_evidence.md` - V190-007 failure and
  partial-success evidence model for the v0.19 actual cancel line. It adds the
  `production-mutation-actual-cancel-failure-evidence` command, consumes
  V190-006 readback reconciliation plus request/response/readback/audit refs,
  classifies recovered, failed, and partial-success outcomes including
  rejected, timeout, unknown, partial fill, already cancelled, venue
  unavailable, and adapter failure, and keeps unknown non-recovered and partial
  fill residual risk visible for Dashboard and release-gate consumption.
- `v0_13_0_no_production_mutation_gate.md` - V130-007 release/PR gate wiring
  for the v0.13 Guarded Live Alpha Preflight line. It aggregates v13 preflight
  evidence and preserves default offline, no-production-mutation, no-listenKey,
  no-Dashboard-order-control, no-real-funds, and no-production-trading
  execution.
- `v0_13_0_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.13.0` Guarded Live Alpha Preflight release.
- `v0_13_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.13.0` GitHub Release. It does not include production
  order submission, order mutation, order-state reads, listenKey lifecycle,
  real funds, production trading, risk/execution-grade live-alpha money math,
  or Dashboard order controls.
- `../evidence/V131-001.md` - dedicated v0.13.0 release-closure evidence with
  exact release commit, hosted Rust Cutover Release Gate run, formal tag,
  GitHub Release URL, publication flags, and tag/main SHA alignment.
- `../evidence/V090-014.md` - dedicated v0.9.0 release-closure evidence with
  exact release commit, hosted gates, formal tag, GitHub Release URL, and
  publication flags.
- `../evidence/V080-009.md` - dedicated v0.8.0 release-closure evidence with
  exact release commit, hosted gates, formal tag, GitHub Release URL, and
  publication flags.
- `../evidence/V080-PRE-004.md` - dedicated v0.7.2 release-closure evidence
  with exact release commit, hosted gates, formal tag, GitHub Release URL, and
  publication flags.
