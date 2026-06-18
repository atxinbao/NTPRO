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
- `v0_9_1_readiness_report.md` - planning/readiness tracker for the v0.9.1
  Strategy Runtime semantics and audit hardening patch. It does not create a
  tag, does not publish a GitHub Release, and keeps Binance testnet order proof
  deferred to v0.10.0.
- `v0_9_1_release_notes.md` - release-note draft for a possible
  owner-approved v0.9.1 Strategy Runtime hardening release. It keeps Binance
  testnet order proof deferred to v0.10.0.
- `../evidence/V090-014.md` - dedicated v0.9.0 release-closure evidence with
  exact release commit, hosted gates, formal tag, GitHub Release URL, and
  publication flags.
- `../evidence/V080-009.md` - dedicated v0.8.0 release-closure evidence with
  exact release commit, hosted gates, formal tag, GitHub Release URL, and
  publication flags.
- `../evidence/V080-PRE-004.md` - dedicated v0.7.2 release-closure evidence
  with exact release commit, hosted gates, formal tag, GitHub Release URL, and
  publication flags.
