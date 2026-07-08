# Release Evidence

Final release checks, benchmark summaries, and audit reports live here.

- `product_live_trading_roadmap.md` - planning baseline for the path from
  v0.20 production order lifecycle evidence through v0.21 unified read model
  and toward v0.22-v0.26+ product-grade live trading readiness. It is the
  current roadmap guardrail for future milestone planning.
- `v0_21_0_unified_read_model_contract.md` - V210 unified account, position,
  order, fill, risk, and lifecycle read-model contract and fail-closed rules.
- `v0_21_0_unified_read_model_schema.json` - machine-readable schema baseline
  for the v0.21 unified read model snapshot contract.
- `v0_21_1_health_status_semantics.md` - V211 patch contract that separates
  component snapshot health, full unified snapshot health, and read-only
  dashboard view degradation semantics.
- `v0_21_1_release_notes.md` - V211 patch release notes draft covering
  health-status hardening, executable read-model projection replay coverage,
  JSON Schema boundary tightening, and the Trader Terminal read-model runtime
  bridge.
- `v0_21_0_account_snapshot_read_model.md` - V210 account snapshot read-model
  component contract for redacted balances, available funds, margin/risk
  summary, account freshness, provenance, and no-operation Dashboard boundary.
- `v0_21_0_position_read_model.md` - V210 position read-model component
  contract for long/short/flat position state, quantity precision, risk
  projection inputs, source freshness, and account-position lineage checks.
- `v0_21_0_order_lifecycle_read_model.md` - V210 order lifecycle read-model
  component contract for submit candidate, attempt ledger, response redaction,
  readback, cancel evidence, audit state, and no-retry boundaries.
- `v0_21_0_fill_execution_read_model.md` - V210 fill/execution read-model
  component contract for fill/execution identity, order linkage, dedupe,
  partial fill visibility, source freshness, and reconciliation boundaries.
- `v0_21_0_risk_state_projection.md` - V210 unified risk state projection for
  account, position, order, and fill rollups, risk-state priority, degradation,
  and no-automatic-action boundaries.
- `v0_21_0_trader_terminal_readonly_dashboard.md` - V210 Trader Terminal
  read-only Dashboard foundation and V211-005 runtime bridge for the canonical
  `v0_21/unified_read_model_snapshot.json` artifact. It displays account,
  position, order, fill, risk, lifecycle, and audit/provenance diagnostics
  without submit, approval, cancel, retry, replace, amend, flatten, or
  product-grade trading terminal claims.
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
- `v0_19_0_dashboard_actual_cancel_audit_view.md` - V190-008 Dashboard
  read-only audit view for the v0.19 actual cancel line. It consumes risk gate,
  owner approval, single-shot cancel attempt, readback reconciliation, and
  failure evidence artifacts, distinguishes ready/recovered/degraded/failed/
  unknown states, and degrades missing, mismatched, unknown, stale, or boundary
  violating evidence without adding Dashboard cancel, approval, retry, bulk, or
  write-operation controls.
- `v0_19_0_actual_cancel_golden_trace_fixtures.md` - V190-009 golden trace and
  fixture coverage for the v0.19 owner-approved actual cancel line. It records
  success, approval missing, approval reused, risk mismatch, adapter
  unsupported, rejected, timeout, unknown, already-cancelled, and partial-fill
  cases with request/response/readback/audit/provenance references and a Rust
  CLI harness, without adding live venue credentials, retry, remediation,
  second cancel, or Dashboard cancel controls.
- `v0_19_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.19.0` GitHub Release. It states that v0.19 includes only
  owner-approved single-shot actual cancel and excludes automatic cancel, bulk
  cancel, Dashboard cancel controls, retry, second cancel, remediation, and
  production order submit lifecycle.
- `v0_19_0_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.19.0` GitHub Release. It records the `v19-release-gates`
  aggregate, golden trace coverage, release-blocking conditions, Dashboard
  read-only boundary, and the handoff that v0.20 enters owner-approved
  production order lifecycle.
- `v0_19_0_release_manifest.json` - machine-readable v0.19.0 release manifest
  for the strict v19 provenance gate. It fixes the published tag, release
  commit/tree, toolchain, release notes/readiness inputs, golden trace
  manifest, v19 gate output root, and strict-manifest artifact path without
  changing the published release tag or runtime behavior.
- `v0_19_1_release_notes.md` - closeout-evidence-complete release notes for the
  v0.19.1 actual-cancel release evidence hardening patch. It records V191-006
  as the V190-004 / PR #598 post-merge review attestation, records V191-007 as
  the standalone v19 release gate hardening that defaults to
  `target/release/nautilus`, and keeps v0.20 production order lifecycle blocked
  until all v0.19.1 closeout issues merge.
- `v0_19_1_readiness_report.md` - closeout-evidence-complete readiness report for
  the v0.19.1 patch. It records the V191 chain, the missing PR #598 GitHub
  review submissions, the compensating post-merge review evidence, the
  standalone gate release-binary boundary, and the no-new-capability boundary.
- `../scope/v0_20_0_owner_approved_production_order_lifecycle_foundation.md` -
  V200-000 scope decision and go/no-go gate for the v0.20 owner-approved
  production order lifecycle foundation. It allows only a bounded
  submit/readback/cancel/audit foundation after v0.19.1 closeout evidence and
  keeps automatic execution, bulk orders, retry/replace/amend/flatten,
  Dashboard order controls, and general production trading platform claims out
  of scope.
- `v0_20_0_order_lifecycle_safety_contract.md` - V200-001 planned safety
  contract for the v0.20 production order lifecycle foundation. It defines the
  lifecycle states, allowed and forbidden transitions, immutable fields,
  evidence fields, failure semantics, submit/readback/cancel/audit boundaries,
  and read-only Dashboard boundary before runtime or adapter implementation.
- `v0_20_0_pre_submit_risk_gate.md` - V200-002 Rust pre-submit risk gate
  contract and local implementation notes. It defines the allow, deny, and
  blocked evidence model, stable rejection codes, required account/instrument/
  venue/order/approval/provenance checks, and keeps actual submit, retry,
  automatic remediation, adapter calls, and Dashboard order controls out of
  scope.
- `v0_20_0_owner_approval_lifecycle.md` - V200-003 Rust owner approval
  lifecycle contract and local implementation notes. It binds approval request
  digest, scope, owner, expiry, nonce, environment, and release provenance to a
  single submit candidate, defines rejected/expired/revoked/consumed evidence,
  and keeps Dashboard approval controls and actual submit out of scope.
- `v0_20_0_signing_material_env_gate.md` - V200-004 Rust signing material
  env-only gate contract and local implementation notes. It blocks missing,
  empty, wrong-environment, or non-env credential material, records only
  redacted env-var/fingerprint evidence, and keeps secret storage, remote key
  management, Dashboard credential UI, and actual submit out of scope.
- `v0_20_0_single_shot_submit_request_builder.md` - V200-005 deterministic
  single-shot production submit request builder contract and local
  implementation notes. It consumes only risk allow, owner approval, and signing
  readiness evidence, emits request digest plus redacted preview, and keeps
  signing, networking, adapter submit, retry, and Dashboard controls out of
  scope.
- `v0_20_0_guarded_single_shot_submit_candidate.md` - V200-006 guarded
  single-shot submit candidate contract and local implementation notes. It
  validates prerequisite evidence, request digest, release provenance, manual
  online gate, and duplicate-attempt state before recording submitted evidence
  and consuming owner approval; preview and dry-run modes remain no-submit.
- `v0_20_0_submit_response_redaction.md` - V200-007 production submit response
  redaction contract and local implementation notes. It consumes submitted
  attempt evidence, emits accepted/rejected/unknown/malformed redacted response
  evidence, retains only readback/audit correlation fields, and forbids raw
  response, header, credential, signature, token, signed-query, signed-URL, and
  Dashboard raw-response exposure.
- `v0_20_0_submit_readback_reconciliation.md` - V200-008 post-submit readback
  reconciliation contract and local implementation notes. It compares local
  submit expectation, V200-007 redacted response evidence, and venue readback
  snapshot fields, emits matched/mismatched/missing/ambiguous/readback_failed
  evidence, and keeps automatic cancel, retry, remediation, raw readback body
  recording, and Dashboard order controls out of scope.
- `v0_20_0_failure_no_retry_evidence.md` - V200-009 failure and no-retry
  evidence model. It records blocked, validation, approval, credential, submit,
  venue rejection, unknown response, readback, cancel-required, and audit
  incomplete failures with stable codes, source evidence pointers, next allowed
  actions, release/Dashboard audit consumability, and no implicit retry or
  automatic remediation.
- `v0_20_0_dashboard_order_lifecycle_audit.md` - V200-010 Dashboard
  read-only production order lifecycle audit view. It consumes guarded submit,
  redacted response, readback reconciliation, failure/no-retry, and audit
  closeout evidence, shows unknown/missing/mismatch as risk-visible, and keeps
  submit, approval, retry, replacement, amendment, flatten, cancel, and
  remediation controls out of the Dashboard.
- `v0_20_0_order_lifecycle_golden_traces.md` - V200-011 executable golden
  traces and fixture coverage for the v0.20 production order lifecycle. It
  covers pre-submit blocking, accepted/rejected/unknown responses, readback
  matched/mismatch/missing, failure/no-retry evidence, Dashboard read-only audit
  references, credential/plaintext boundaries, and release-scope replay wiring.
- `v0_20_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.20.0` GitHub Release. It states that v0.20 includes only
  the Owner-Approved Production Order Lifecycle Foundation and excludes
  product-grade live trading, implicit retry, automatic cancel/remediation,
  bulk order execution, strategy-driven production execution, multi-account or
  multi-venue execution, and Dashboard operation controls.
- `v0_20_0_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.20.0` GitHub Release. It records the V200 issue chain,
  `v20-release-gates`, `v20-strict-provenance`, golden trace coverage, release
  blocking conditions, Dashboard read-only boundary, and strict provenance
  closure.
- `v0_20_0_release_manifest.json` - machine-readable v0.20.0 release manifest
  for the strict v20 provenance gate. It fixes the planned tag, release notes,
  readiness report, golden trace manifest, gate list, boundary flags, and
  strict-manifest artifact path while leaving commit/tree/binary hash fields to
  `scripts/ai/verify_release_strict.sh v20`.
- `../evidence/V200-012.md` - V200-012 release gate and strict provenance
  evidence. It records the v20 gate wiring, publication guard scope, GitHub
  issue state, local validation commands, and rollback plan for the formal
  `ntpro-rust-only-v0.20.0` GitHub Release.
- `v0_20_1_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.20.1` GitHub Release. It states that v0.20.1 is a
  hardening patch only and does not expand production submit capability.
- `v0_20_1_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.20.1` GitHub Release. It records the V201 evidence chain,
  v0.20.1 gate, publication guard, current-surface guard, and v0.21.0
  dependency proof.
- `v0_20_1_release_manifest.json` - machine-readable v0.20.1 patch release
  manifest for the V201 gate. It fixes the V201 evidence set, base v0.20.0
  release dependency, planned patch tag, boundary flags, release body path, and
  v0.21.0 dependency sources.
- `v0_21_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.21.0` GitHub Release. It records the Unified Read Model
  Foundation scope and preserves no new submit capability, no Dashboard
  operation controls, and no product-grade trading terminal claim.
- `v0_21_0_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.21.0` GitHub Release. It records the V210 evidence chain,
  read-model golden trace scope, v21 release gates, and strict provenance
  requirements.
- `v0_21_0_release_manifest.json` - machine-readable v0.21.0 release manifest
  for the V210 gate. It fixes the read-model evidence set, base v0.20.1
  release dependency, planned release tag, boundary flags, release body path,
  golden trace manifest, and strict provenance target.
- `v0_21_1_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.21.1` GitHub Release. It records the Unified Read Model
  Foundation Hardening Patch scope, V211 release gates, v0.22.0 dependency
  proof, and the no-workbench/no-submit boundary.
- `v0_21_1_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.21.1` GitHub Release. It records V211 evidence,
  v21.1 release gates, strict provenance, publication/current-surface guards,
  and v0.22.0 dependency sources.
- `v0_21_1_release_manifest.json` - machine-readable v0.21.1 patch release
  manifest for the V211 gate. It fixes the V211 evidence set, v0.21.0 closeout
  dependency, planned patch tag, boundary flags, release body path, and v0.22.0
  dependency sources.
- `v0_22_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.22.0` GitHub Release. It records the Trader Terminal
  Workbench scope, read-only-first boundary, gated-operation boundary, V220
  evidence chain, release gates, and strict provenance.
- `v0_22_0_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.22.0` GitHub Release. It records V220 evidence, workbench
  evidence inputs, issue closeout requirements, v22 release gates, and strict
  provenance requirements.
- `v0_22_0_release_manifest.json` - machine-readable v0.22.0 release manifest
  for the V220 gate. It fixes the V220 evidence set, v0.21.1 base release
  dependency, planned release tag, workbench evidence paths, boundary flags,
  release body path, and strict provenance target.
- `v0_22_1_release_closeout_evidence.md` - V221-001 release closeout ledger
  for the completed `ntpro-rust-only-v0.21.1` and
  `ntpro-rust-only-v0.22.0` releases. It records live GitHub Release, tag,
  hosted release gate, issue closeout, milestone closeout, and the v0.22.0
  Workbench/runtime-bridge boundary before v0.22.1 hardening begins.
- `v0_22_1_required_false_runtime_boundary.md` - V221-002 required-false
  runtime boundary note. It records the stricter Workbench operation boundary:
  missing or true operation/control fields fail closed, while explicit false
  fields remain healthy.
- `v0_22_1_read_model_executable_replay.md` - V221-003 read-model executable
  replay expansion note. It records 28 executable read-model replay rows, 4
  remaining read-model schema-only rows, and the Workbench/runtime bridge
  wording boundary.
- `v0_22_1_gate_before_publish.md` - V221-004 gate-before-publish release
  governance note. It records the controlled publication sequence: draft
  preparation is allowed before the hosted release gate, but public GitHub
  Release publication requires a successful `Rust Cutover Release Gate` run for
  the same tag commit.
- `v0_22_1_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.22.1` hardening patch. It records required-false runtime
  boundary hardening, expanded executable replay evidence, Workbench render
  smoke, gate-before-publish governance, and strict provenance.
- `v0_22_1_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.22.1` hardening patch. It records V221 evidence,
  v22.1 release gates, strict provenance, publication-order requirements, and
  the continued hard block on `v0.23.0`.
- `v0_22_1_release_manifest.json` - machine-readable v0.22.1 patch release
  manifest for the V221 gate. It fixes the V221 evidence set, v0.22.0 base
  release dependency, planned patch tag, boundary flags, read-model replay
  counts, publication governance, and v0.23.0 dependency sources.
- `../scope/v0_23_0_multi_node_isolation_scope.md` - V230-000 intake scope
  gate for v0.23.0. It records the v0.22.1 release evidence that satisfies the
  hard blocker, the #711-#718 issue order, and the no-submit/no-operation
  boundary before any multi-node implementation begins.
- `v0_23_0_multi_node_isolation_contract.md` - V230-001 contract for v0.23.0
  multi-account, multi-strategy, and multi-venue node isolation. It defines
  `account_key`, `strategy_key`, `venue_node_key`, `isolation_scope_key`, read
  paths, future owner-approved control paths, evidence requirements, and
  forbidden release claims before implementation begins.
- `v0_23_0_isolation_contract_manifest.json` - machine-checkable V230-001
  manifest that maps #713-#718 to required contract sections and validation
  markers.
- `v0_23_0_multi_account_read_model_partitioning.md` - V230-002 release note
  for account identity and read-model partitioning. It records executable
  replay cases for isolated accounts, cross-account mismatch fail-closed
  behavior, and unknown account identity fail-closed behavior.
- `v0_23_0_multi_strategy_supervisor_isolation.md` - V230-003 release note
  for strategy supervisor identity and isolation. It records executable replay
  cases for isolated strategies, cross-strategy mismatch fail-closed behavior,
  and unknown strategy identity fail-closed behavior.
- `v0_23_0_multi_venue_node_lifecycle_boundary.md` - V230-004 release note for
  venue node registry and lifecycle isolation. It records the registry contract
  shape and executable replay cases for isolated nodes, cross-node mismatch
  fail-closed behavior, and unknown venue node identity fail-closed behavior.
- `v0_23_0_multi_node_orchestration_control_plane_gating.md` - V230-005
  release note for orchestration control-plane gating. It records executable
  replay cases for scoped intents, cross-scope route mismatch, shared approval
  consumption, and missing isolation scope fail-closed behavior.
- `../evidence/V230-005.md` - V230-005 orchestration control-plane gate
  evidence for GitHub issue #716.
- `v0_23_0_dashboard_observability_surface.md` - V230-006 release note for the
  Dashboard / Workbench observability surface. It records executable replay and
  render smoke coverage for read-only multi-account, multi-strategy, and
  multi-venue node aggregation, scoped filtering, missing identity degradation,
  and forbidden operation controls.
- `../evidence/V230-006.md` - V230-006 Dashboard observability evidence for
  GitHub issue #717.
- `v0_23_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.23.0` GitHub Release. It records multi-account,
  multi-strategy, and multi-venue node isolation, read-only observability,
  release gates, strict provenance, and the no-submit/no-Dashboard-controls
  boundary.
- `v0_23_0_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.23.0` release. It records V230 evidence, v23 release
  gates, strict provenance, publication-order requirements, and next-track
  boundaries.
- `v0_23_0_release_manifest.json` - machine-readable v0.23.0 release manifest
  for V230 release gates. It fixes the evidence chain, release gate commands,
  read-model replay counts, boundary flags, and strict provenance target.
- `v0_23_1_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.23.1` release. It records V231 evidence, v0.23.0
  dependency proof, v23.1 release gates, strict provenance, and the v0.24.0
  start gate.
- `v0_23_1_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.23.1` GitHub Release. It records the patch closeout
  scope, V231 evidence, publication governance, strict provenance, and no new
  trading capability.
- `v0_23_1_release_manifest.json` - machine-readable v0.23.1 release manifest
  for V231 release gates. It fixes V231 evidence paths, release gate commands,
  boundary flags, publication governance, current release surface guard fields,
  and the v0.24.0 start gate.
- `v0_24_0_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.24.0` release. It records V240 evidence, v24 release
  gates, strict provenance, publication governance, and the v0.25.0 start
  boundary.
- `v0_24_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.24.0` GitHub Release. It records the preview-only
  order-control foundation, V240 evidence, no-submit/no-Dashboard-controls
  boundary, and strict provenance.
- `v0_24_0_release_manifest.json` - machine-readable v0.24.0 release manifest
  for V240 release gates. It fixes V240 evidence paths, release gate commands,
  boundary flags, publication governance, current release surface guard fields,
  and the v0.25.0 start boundary.
- `v0_24_1_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.24.1` release. It records V241 evidence, v24.1 release
  gates, strict provenance, publication governance, and the v0.25.0 start
  boundary.
- `v0_24_1_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.24.1` GitHub Release. It records the v0.24.0
  post-release governance and Dashboard evidence hardening patch, no-submit /
  no-Dashboard-controls boundary, and strict provenance.
- `v0_24_1_release_manifest.json` - machine-readable v0.24.1 release manifest
  for V241 release gates. It fixes V241 evidence paths, release gate commands,
  boundary flags, publication governance, current release surface guard fields,
  and the v0.25.0 start boundary.
- `v0_25_0_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.25.0` release. It records V250 evidence, v25 release
  gates, strict provenance, publication governance, and the v0.26.0 start
  boundary.
- `v0_25_0_release_closeout_evidence.md` - post-release closeout evidence for
  the formal `ntpro-rust-only-v0.25.0` GitHub Release. It records the release
  URL, published timestamp, tag SHA, hosted release gate, publish workflow,
  V250 issue closeout, corrective #804/#805 scope, milestone closeout, and
  no-submit/no-Dashboard-controls boundary.
- `v0_25_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.25.0` GitHub Release. It records the monitoring /
  incident / DR foundation, V250 evidence, no-submit/no-Dashboard-controls
  boundary, gate-before-publish governance, and strict provenance.
- `v0_25_0_release_manifest.json` - machine-readable v0.25.0 release manifest
  for V250 release gates. It fixes V250 evidence paths, release gate commands,
  boundary flags, publication governance, current release surface guard fields,
  and the v0.26.0 start boundary.
- `v0_25_1_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.25.1` release. It records V251 evidence, v25.1 release
  gates, strict provenance, publication governance, and the v0.26.0 start
  boundary.
- `v0_25_1_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.25.1` GitHub Release. It records the v0.25.0
  post-release governance and evidence hardening patch, no-submit /
  no-Dashboard-controls boundary, gate-before-publish governance, and strict
  provenance.
- `v0_25_1_release_manifest.json` - machine-readable v0.25.1 release manifest
  for V251 release gates. It fixes V251 evidence paths, release gate commands,
  boundary flags, publication governance, current release surface guard fields,
  and the v0.26.0 start boundary.
- `v0_26_0_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.26.0` release. It records V260 evidence, v26 release
  gates, strict provenance, publication governance, Dashboard/admin smoke, and
  the v0.27.0 start boundary.
- `v0_26_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.26.0` GitHub Release. It records the Product Hardening
  Foundation, V260 evidence, no-submit/no-Dashboard-controls boundary,
  gate-before-publish governance, and strict provenance.
- `v0_26_0_release_manifest.json` - machine-readable v0.26.0 release manifest
  for V260 release gates. It records V260 evidence paths, release gate
  commands, boundary flags, publication governance, current release surface
  guard fields, and the v0.27.0 start boundary.
- `v0_26_1_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.26.1` closeout patch. It records V261 evidence, v26.1
  release gates, strict provenance, publication governance, and the v0.27.0
  hard-block until v0.26.1 publication evidence exists.
- `v0_26_1_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.26.1` GitHub Release. It records the v0.26.0
  post-publication governance and evidence hardening patch, no-submit /
  no-Dashboard-controls boundary, gate-before-publish governance, and strict
  provenance.
- `v0_26_1_release_manifest.json` - machine-readable v0.26.1 release manifest
  for V261 release gates. It records V261 evidence paths, release gate
  commands, boundary flags, publication governance, current release surface
  guard fields, and the v0.27.0 start boundary.
- `v0_27_0_readiness_report.md` - released readiness report for the formal
  `ntpro-rust-only-v0.27.0` release. It records V270 evidence, v27 release
  gates, strict provenance, publication governance, source-tree provenance, and
  the no-submit/no-Dashboard/Admin-trading-controls boundary.
- `v0_27_0_release_notes.md` - release notes for the formal
  `ntpro-rust-only-v0.27.0` GitHub Release. It records the Product Operations
  Runtime Integration Foundation, V270 evidence, no-submit/no-Dashboard/Admin-
  trading-controls boundary, gate-before-publish governance, and strict
  provenance.
- `v0_27_0_release_manifest.json` - machine-readable v0.27.0 release manifest
  for V270 release gates. It records V270 evidence paths, release gate
  commands, boundary flags, publication governance, current release surface
  guard fields, and the v0.28.0 start boundary.
- `v0_27_0_release_closeout_evidence.md` - post-publication closeout evidence
  for the formal `ntpro-rust-only-v0.27.0` release. It records the public
  GitHub Release, hosted release gate run `28921344889`, tag commit,
  release body/source hash match, generated publication evidence policy,
  `v0.27.0` milestone closeout, and the v0.28.0 hard-block until v0.27.1
  publication evidence exists.
- `v0_26_0_intake_gate.md` - V260-000 intake proof that all V251 issues are
  closed, the `ntpro-rust-only-v0.25.1` GitHub Release was published after
  hosted gate success, and v0.26.0 starts only as a gated product hardening
  foundation track with no inherited operation controls.
- `v0_27_0_intake_gate.md` - V270-000 intake proof that all V261 contractual
  issues are closed, the `ntpro-rust-only-v0.26.1` GitHub Release was
  published after hosted gate success, and v0.27.0 starts only as a gated
  Product Operations Runtime Integration Foundation track with no inherited
  operation controls.
- `v0_27_0_product_operations_runtime_integration_boundary_contract.md` -
  V270-001 contract for the v0.27.0 Product Operations Runtime Integration
  Foundation scope. It separates allowed read/admin integration surfaces from
  forbidden trading controls, requires provenance/freshness/redaction/lineage
  semantics, and blocks production execution runtime or product-grade live
  trading terminal claims.
- `v0_27_0_external_identity_permission_foundation.md` - V270-002 contract for
  external identity-provider evidence and permission mapping foundation. It
  records IdP provenance, role mapping freshness/redaction/lineage, v26
  permission-boundary alignment, and required-false trading permissions without
  opening runtime SSO/IAM authorization or trading controls.
- `v0_27_0_persistent_operation_audit_storage_foundation.md` - V270-003
  contract for append-only persistent operation audit storage foundation. It
  records audit sink provenance, sequence/hash lineage, redaction, retention
  metadata, and store/source reconciliation without operation execution,
  remediation, adapter send, live exchange request, or Dashboard controls.
- `v0_27_0_deployment_orchestration_foundation.md` - V270-004 contract for
  preview-first deployment, upgrade, rollback, and post-check orchestration
  state. It requires owner approval, release gate evidence, fresh environment
  provenance, and rollback plan lineage without automatic deploy, rollback,
  remediation, adapter send, live exchange request, or Dashboard controls.
- `v0_27_0_long_run_telemetry_slo_runtime_evidence.md` - V270-005 contract for
  long-run telemetry ingestion and SLO runtime evidence. It validates source
  contracts, freshness, redaction, sampling windows, gaps, SLO rollups, and
  read-only Admin Workbench / Dashboard degradation reasons without automatic
  remediation, retry scheduling, adapter send, live exchange request, or
  trading controls.
- `v0_27_0_admin_workbench_runtime_state_bridge.md` - V270-006 contract for
  the Admin Workbench / Dashboard runtime state bridge. It renders identity,
  audit storage, deployment orchestration, telemetry/SLO, and runtime boundary
  evidence as a read-only/admin surface while malformed, stale, missing,
  unredacted, drifted, or control-enabled artifacts degrade or fail closed.
- `v0_27_0_runtime_integration_fail_closed_hardening.md` - V270-007 contract
  for shared runtime integration fail-closed/no-trading-control hardening. It
  validates downgrade vs fail-closed rules across identity, audit,
  orchestration, telemetry, and Admin Workbench bridge artifacts, and blocks
  forbidden controls, missing required-false boundaries, and product-ready
  claims.
- `v0_26_0_product_hardening_boundary_contract.md` - V260-001 contract for the
  v0.26.0 product hardening foundation scope. It records allowed hardening
  areas, required-false trading boundary flags, fail-closed rules, and the
  non-goal of product-grade live trading terminal readiness.
- `v0_26_0_operator_permission_model.md` - V260-002 operator permission model
  and role boundary evidence for viewer, operator, release gatekeeper,
  incident owner, and auditor roles. It keeps permission evidence read-only and
  does not add SSO/IAM integration or live operation authorization.
- `v0_26_0_operation_audit_trail.md` - V260-003 operation audit trail and
  immutable action evidence model for operator ack, runbook decision, release
  gate action, permission denial, and rollback recommendation events. It keeps
  audit evidence read-only and does not execute operation intents or add a live
  control API.
- `v0_26_0_deployment_provenance_model.md` - V260-004 deployment topology and
  environment provenance model for local/dev/staging/prod-like evidence. It
  records artifact digest, config source, release tag, node scope, and runtime
  boundary without production deploy automation or real production trading
  readiness claims.
- `v0_26_0_upgrade_rollback_runbook_evidence.md` - V260-005 upgrade, rollback,
  and release operation runbook evidence for preview, blocked preview, rollback
  recommendation, preflight, approval, post-check, audit lineage, release gate,
  and Dashboard read-only evidence. It does not execute deployment, rollback,
  release publication, trading operation, or automatic remediation.
- `v0_26_0_slo_runbook_stability_evidence.md` - V260-006 SLO, runbook, and
  long-run stability evidence for sample provenance, freshness, redaction,
  component coverage, error budget, restart recommendation, runbook staleness,
  and release drift. It is release-gate and Dashboard read-only evidence only;
  it does not execute automatic recovery, restart, strategy stop, cancel, order
  submit, trading recovery, or remediation.
- `v0_26_0_dashboard_admin_boundary_surface.md` - V260-007 Dashboard / Trader
  Terminal product hardening read-only admin surface for permission boundary,
  operation audit, deployment provenance, upgrade/rollback preview, and
  stability/SLO evidence. It adds provenance drill-down and degraded/fail-closed
  diagnostics without operation buttons, live control API, trading controls, or
  automatic remediation.
- `../evidence/V260-008.md` - V260-008 v26 release gate, strict provenance, and
  release replay evidence for the Product Hardening Foundation.
- `v0_23_0_evidence_replay_only_boundary.md` - V231-004 boundary note that
  keeps v0.23.x framed as evidence/replay/read-only observability only, not as
  a production multi-node runtime or product-grade terminal implementation.
- `v0_23_0_publication_evidence_audit_path.md` - V231-005 source-tree plus
  GitHub remote reconstruction path for v0.23.0 publication evidence. It keeps
  generated local publication JSON from being the sole audit source.
- `../evidence/V230-007.md` - V230-007 release gate, strict provenance, and
  v0.23.0 publication evidence ledger for GitHub issue #718 and the
  `ntpro-rust-only-v0.23.0` GitHub Release.
- `../scope/v0_22_0_trader_terminal_workbench_scope.md` - V220-000 scope
  decision for the v0.22.0 Trader Terminal workbench line. It records the
  v0.21.1 closeout dependency, read-only-first workbench boundary, gated manual
  operation entry design, and no-ungated-submit/cancel/retry/replace/amend/
  flatten/product-grade-terminal claim.
- `../evidence/V220-000.md` - V220-000 scope decision and v0.21.1 dependency
  gate evidence for GitHub issue #683.
- `v0_22_0_trader_terminal_workbench_shell.md` - V220-001 Trader Terminal
  workbench shell release note. It records the read-only dashboard shell,
  navigation, status summary, artifact/provenance drawer, degraded missing
  read-model fallback, and no operation buttons/product-grade-terminal claim.
- `../evidence/V220-001.md` - V220-001 Trader Terminal read-only workbench
  shell and navigation evidence for GitHub issue #684.
- `v0_22_0_account_position_workbench_panels.md` - V220-002 Trader Terminal
  account and position panel release note. It records read-only account status,
  balance, position side, quantity, notional, precision, freshness,
  provenance, redaction, and lineage projection without funds-transfer,
  account-mutation, auto-flatten, or position-repair controls.
- `../evidence/V220-002.md` - V220-002 Trader Terminal account and position
  workbench panel evidence for GitHub issue #685.
- `v0_22_0_order_fill_workbench_panels.md` - V220-003 Trader Terminal order
  and fill panel release note. It records read-only order lifecycle, attempt
  ledger, readback, audit, fill/execution identity, dedupe, partial fill,
  linkage, reconciliation, risk input, provenance, lineage, schema-only truth,
  and no order/fill operation controls.
- `../evidence/V220-003.md` - V220-003 Trader Terminal order and fill
  workbench panel evidence for GitHub issue #686.
- `v0_22_0_risk_alert_audit_provenance_workbench_panels.md` - V220-004 Trader
  Terminal risk, alerts, audit, and provenance drill-down panel release note.
  It records read-only risk priority, alert severity, audit evidence
  completeness, release provenance, artifact digest, lineage diagnostics, and
  no automatic risk/alert/audit/provenance action controls.
- `../evidence/V220-004.md` - V220-004 Trader Terminal risk, alerts, audit,
  and provenance drill-down panel evidence for GitHub issue #687.
- `v0_22_0_gated_manual_operation_entry_contract.md` - V220-005 Trader
  Terminal gated manual operation entry contract release note. It records the
  disabled/gated intent preview, owner approval reference, risk decision
  reference, audit evidence reference, blocked states, fail-closed ungated
  attempt behavior, and no submit/cancel/retry/replace/amend/flatten controls.
- `../evidence/V220-005.md` - V220-005 gated manual operation entry contract
  evidence for GitHub issue #688.
- `v0_22_0_runtime_degradation_boundary_tests.md` - V220-006 Trader Terminal
  runtime degradation and boundary test release note. It records missing,
  stale, schema mismatch, component unavailable, redaction breach, provenance
  mismatch, forbidden-control, read-only-first, and no-product-grade-claim gate
  coverage.
- `../evidence/V220-006.md` - V220-006 runtime degradation and boundary test
  evidence for GitHub issue #689.
- `../evidence/V220-007.md` - V220-007 release gate, strict provenance, and
  workbench evidence closeout for GitHub issue #690 and the
  `ntpro-rust-only-v0.22.0` GitHub Release.
- `../evidence/V221-001.md` - V221-001 release closeout milestone and evidence
  ledger for GitHub issue #705. It records closed v0.21.1/v0.22.0 milestones,
  published release/tag facts, hosted release gate success, and the blocker
  chain into v0.22.1 and v0.23.0.
- `../evidence/V221-002.md` - V221-002 required-false runtime operation
  boundary evidence for GitHub issue #706. It records missing/false/true
  runtime health tests and the `new_submit_capability` dashboard status field.
- `../evidence/V221-003.md` - V221-003 executable read-model replay expansion
  evidence for GitHub issue #707. It records promoted position, fill, order,
  risk, and dashboard forbidden-controls rows plus the remaining schema-only
  read-model rows.
- `../evidence/V221-004.md` - V221-004 gate-before-publish release governance
  evidence for GitHub issue #708. It records the release publication entrypoint
  and the hosted gate proof required before public release publication.
- `../evidence/V221-005.md` - V221-005 Workbench render smoke and read-only
  regression evidence for GitHub issue #709. It records the deterministic
  `read_model_runtime` fixture, Dashboard renderer smoke, and absent operation
  surface assertions.
- `../evidence/V221-006.md` - V221-006 release gate, strict provenance, and
  v0.22.1 publication evidence ledger for GitHub issue #710 and the
  `ntpro-rust-only-v0.22.1` GitHub Release.
- `../evidence/V211-006.md` - V211-006 release gate, strict provenance, and
  dependency proof evidence for the `ntpro-rust-only-v0.21.1` GitHub Release.
- `../evidence/V211-001.md` - V211-001 release closeout and milestone evidence
  backfill for the published `ntpro-rust-only-v0.21.0` GitHub Release. It
  records the release URL, tag commit, hosted release gate run, closed V210
  issues, closed milestone #8, and the v0.21.1 hard blocker before v0.22.0.
- `../evidence/V210-008.md` - dedicated V210-008 release gate and strict
  provenance evidence for the `ntpro-rust-only-v0.21.0` GitHub Release.
- `../evidence/V201-007.md` - V201-007 release gate and dependency proof
  evidence. It records v0.20.1 release gate wiring, publication guard scope,
  GitHub milestone dependency proof, local validation commands, and rollback
  plan for the `ntpro-rust-only-v0.20.1` GitHub Release.
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
