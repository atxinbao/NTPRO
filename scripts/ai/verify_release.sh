#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

FEATURES="${NAUTILUS_RUST_FEATURES:-arrow,ffi,high-precision,streaming,defi}"
export REQUIRE_GOLDEN_REPLAY="${REQUIRE_GOLDEN_REPLAY:-1}"

NAUTILUS_RELEASE_BIN="$ROOT/target/release/nautilus"
NTPRO_NODE_RELEASE_BIN="$ROOT/target/release/ntpro-node"

require_help_contains() {
  local file="$1"
  shift
  local pattern
  for pattern in "$@"; do
    if ! grep -Eiq -- "$pattern" "$file"; then
      echo "help output missing required pattern '$pattern' in $file" >&2
      cat "$file" >&2
      exit 1
    fi
  done
}

run_full_checks() {
  echo "== verify_release: full checks =="
  scripts/ai/verify_full.sh
}

release_cli_feature_args() {
  local selected=()
  local feature

  IFS=',' read -r -a feature_list <<< "$FEATURES"
  for feature in "${feature_list[@]}"; do
    feature="${feature//[[:space:]]/}"
    if [[ "$feature" == "defi" ]]; then
      selected+=("$feature")
    fi
  done

  if (( ${#selected[@]} > 0 )); then
    printf '%s\n' "--features"
    (IFS=','; printf '%s\n' "${selected[*]}")
  fi
}

ensure_release_cli_binaries() {
  echo "== verify_release: release CLI binary build =="
  local cli_feature_args=()
  while IFS= read -r arg; do
    cli_feature_args+=("$arg")
  done < <(release_cli_feature_args)

  cargo build -p nautilus-cli --release --bin nautilus --bin ntpro-node "${cli_feature_args[@]}"

  if [[ ! -x "$NAUTILUS_RELEASE_BIN" ]]; then
    echo "missing release nautilus binary: $NAUTILUS_RELEASE_BIN" >&2
    exit 1
  fi
  if [[ ! -x "$NTPRO_NODE_RELEASE_BIN" ]]; then
    echo "missing release ntpro-node binary: $NTPRO_NODE_RELEASE_BIN" >&2
    exit 1
  fi
}

run_release_build_product_surface() {
  ensure_release_cli_binaries

  echo "== verify_release: Rust CLI product surface =="
  if ! cargo metadata --no-deps --format-version=1 | grep -q '"name":"nautilus-cli"'; then
    echo "nautilus-cli package is missing" >&2
    exit 1
  fi

  "$NAUTILUS_RELEASE_BIN" --help >/tmp/nautilus_cli_help.txt
  require_help_contains /tmp/nautilus_cli_help.txt \
    'backtest' \
    'live' \
    'sandbox' \
    'data' \
    'supervisor'

  "$NAUTILUS_RELEASE_BIN" supervisor --help >/tmp/nautilus_supervisor_help.txt
  require_help_contains /tmp/nautilus_supervisor_help.txt \
    'register' \
    'list' \
    'start' \
    'stop' \
    'pause' \
    'resume' \
    'reconnect-data' \
    'reconnect-execution' \
    'status' \
    'logs' \
    'metrics' \
    'sandbox-only'

  "$NTPRO_NODE_RELEASE_BIN" --help >/tmp/ntpro_node_help.txt
  require_help_contains /tmp/ntpro_node_help.txt \
    'config' \
    'run-id' \
    'output' \
    'stop-file' \
    'sandbox-only'
}

run_rust_only_gates() {
  echo "== verify_release: Rust-only runtime check =="
  scripts/ai/check_rust_only_runtime.sh

  echo "== verify_release: final Cython removed check =="
  scripts/ai/check_cython_removed.sh
}

run_v02_supervisor_smoke() {
  echo "== verify_release: v0.2 two-node supervisor smoke =="
  if [[ "${NTPRO_V02_009_SKIP_BUILD:-0}" == "1" ]]; then
    echo "release gate must not set NTPRO_V02_009_SKIP_BUILD=1" >&2
    exit 1
  fi
  NTPRO_RELEASE_GATE=1 NTPRO_V02_009_SKIP_BUILD=0 scripts/ai/v02_two_node_supervisor_smoke.sh
}

run_v03_supervisor_control_smoke() {
  echo "== verify_release: v0.3 supervisor control smoke =="
  ensure_release_cli_binaries
  NTPRO_V03_CONTROL_SKIP_BUILD=1 \
    NTPRO_V03_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V03_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/v03_supervisor_control_smoke.sh
}

run_v03_dashboard_smoke() {
  echo "== verify_release: v0.3 dashboard control smoke =="
  ensure_release_cli_binaries
  NTPRO_V03_010_SKIP_BUILD=1 \
    NTPRO_V03_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V03_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/v03_dashboard_smoke.sh
}

run_v05_workflow_artifacts_smoke() {
  echo "== verify_release: v0.5 workflow artifact smoke =="
  ensure_release_cli_binaries
  NTPRO_V05_WORKFLOW_SKIP_BUILD=1 \
    NTPRO_V05_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v05_workflow_artifacts.sh
}

run_v06_binance_testnet_dry_run_smoke() {
  echo "== verify_release: v0.6 Binance testnet dry-run smoke =="
  ensure_release_cli_binaries
  NTPRO_V06_SKIP_BUILD=1 \
    NTPRO_V06_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v06_binance_testnet_dry_run.sh
}

run_v07_default_offline_gate() {
  echo "== verify_release: v0.7 default offline gate =="
  ensure_release_cli_binaries
  NTPRO_V07_SKIP_BUILD=1 \
    NTPRO_V07_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v07_default_offline_gate.sh
}

run_v07_manual_online_preflight() {
  echo "== verify_release: v0.7 manual online preflight =="
  ensure_release_cli_binaries
  NTPRO_V07_SKIP_BUILD=1 \
    NTPRO_V07_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v07_manual_online_gate.sh
}

run_v08_default_offline_gate() {
  echo "== verify_release: v0.8 default offline synthetic secret leak gate =="
  ensure_release_cli_binaries
  NTPRO_V08_SKIP_BUILD=1 \
    NTPRO_V08_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v08_default_offline_gate.sh
}

run_v08_authenticated_readonly_preflight() {
  echo "== verify_release: v0.8 authenticated read-only preflight =="
  ensure_release_cli_binaries
  NTPRO_V08_SKIP_BUILD=1 \
    NTPRO_V08_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v08_authenticated_readonly_gate.sh
}

run_v09_strategy_runtime_smoke() {
  echo "== verify_release: v0.9 strategy runtime smoke =="
  ensure_release_cli_binaries
  NTPRO_V09_SKIP_BUILD=1 \
    NTPRO_V09_NTPRO_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/verify_v09_strategy_runtime_smoke.sh
}

run_v09_shadow_mode_no_order_gate() {
  echo "== verify_release: v0.9 shadow-mode no-order gate =="
  ensure_release_cli_binaries
  NTPRO_V09_SKIP_BUILD=1 \
    NTPRO_V09_NTPRO_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/verify_v09_shadow_mode_no_order_gate.sh
}

run_v091_strategy_supervisor_dashboard_integration() {
  echo "== verify_release: v0.9.1 strategy supervisor/dashboard integration =="
  ensure_release_cli_binaries
  NTPRO_V091_SKIP_BUILD=1 \
    NTPRO_V091_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V091_NTPRO_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/verify_v091_strategy_supervisor_dashboard_integration.sh
}

run_v10_offline_release_gates() {
  echo "== verify_release: v0.10 offline release gates =="
  ensure_release_cli_binaries
  NTPRO_V10_SKIP_BUILD=1 \
    NTPRO_V10_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v10_offline_release_gates.sh
}

run_v10_manual_order_proof_preflight() {
  echo "== verify_release: v0.10 manual order proof preflight =="
  ensure_release_cli_binaries
  NTPRO_V10_SKIP_BUILD=1 \
    NTPRO_V10_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v10_manual_order_proof_gate.sh
}

run_v11_offline_release_gates() {
  echo "== verify_release: v0.11 offline release gates =="
  ensure_release_cli_binaries
  NTPRO_V11_SKIP_BUILD=1 \
    NTPRO_V11_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v11_offline_release_gates.sh
}

run_v12_offline_release_gates() {
  echo "== verify_release: v0.12 offline release gates =="
  ensure_release_cli_binaries
  NTPRO_V12_SKIP_BUILD=1 \
    NTPRO_V12_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v12_offline_release_gates.sh
}

run_v12_manual_online_preflight() {
  echo "== verify_release: v0.12 manual online preflight =="
  ensure_release_cli_binaries
  NTPRO_V12_SKIP_BUILD=1 \
    NTPRO_V12_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v12_manual_online_preflight.sh
}

run_v13_no_production_mutation_gate() {
  echo "== verify_release: v0.13 no-production-mutation gate =="
  ensure_release_cli_binaries
  NTPRO_V13_SKIP_BUILD=1 \
    NTPRO_V13_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V12_SKIP_BUILD=1 \
    NTPRO_V12_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v13_no_production_mutation_gate.sh
}

run_v14_release_gates() {
  echo "== verify_release: v0.14 release gates =="
  ensure_release_cli_binaries
  NTPRO_V14_SKIP_BUILD=1 \
    NTPRO_V14_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V14_NTPRO_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/verify_v14_release_gates.sh
}

run_v15_release_gates() {
  echo "== verify_release: v0.15 release gates =="
  ensure_release_cli_binaries
  NTPRO_V15_SKIP_BUILD=1 \
    NTPRO_V15_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v15_release_gates.sh
}

run_v151_release_gates() {
  echo "== verify_release: v0.15.1 release gates =="
  ensure_release_cli_binaries
  NTPRO_V151_SKIP_BUILD=1 \
    NTPRO_V151_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V151_NTPRO_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/verify_v151_release_gates.sh
}

run_v16_release_gates() {
  echo "== verify_release: v0.16 release gates =="
  ensure_release_cli_binaries
  NTPRO_V16_SKIP_BUILD=1 \
    NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V16_NTPRO_NODE_BIN="$NTPRO_NODE_RELEASE_BIN" \
    scripts/ai/verify_v16_release_gates.sh
}

run_v17_release_gates() {
  echo "== verify_release: v0.17 release gates =="
  ensure_release_cli_binaries
  NTPRO_V17_SKIP_BUILD=1 \
    NTPRO_V17_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V16_SKIP_BUILD=1 \
    NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v17_release_gates.sh
}

run_v18_release_gates() {
  echo "== verify_release: v0.18 release gates =="
  ensure_release_cli_binaries
  NTPRO_V18_SKIP_BUILD=1 \
    NTPRO_V18_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V17_SKIP_BUILD=1 \
    NTPRO_V17_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    NTPRO_V16_SKIP_BUILD=1 \
    NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v18_release_gates.sh
}

run_v19_release_gates() {
  echo "== verify_release: v0.19 release gates =="
  ensure_release_cli_binaries
  NTPRO_V19_SKIP_BUILD=1 \
    NTPRO_V19_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v19_release_gates.sh
}

run_v20_release_gates() {
  echo "== verify_release: v0.20 release gates =="
  ensure_release_cli_binaries
  NTPRO_V20_SKIP_BUILD=1 \
    NTPRO_V20_NAUTILUS_BIN="$NAUTILUS_RELEASE_BIN" \
    scripts/ai/verify_v20_release_gates.sh
}

run_v20_patch_release_gates() {
  echo "== verify_release: v0.20.1 patch release gates =="
  scripts/ai/verify_v20_patch_release_gates.sh
}

run_v21_read_model_contract() {
  echo "== verify_release: v0.21 read model contract =="
  scripts/ai/verify_v21_read_model_contract.sh
}

run_v21_account_snapshot_read_model() {
  echo "== verify_release: v0.21 account snapshot read model =="
  scripts/ai/verify_v21_account_snapshot_read_model.sh
}

run_v21_position_read_model() {
  echo "== verify_release: v0.21 position read model =="
  scripts/ai/verify_v21_position_read_model.sh
}

run_v21_order_lifecycle_read_model() {
  echo "== verify_release: v0.21 order lifecycle read model =="
  scripts/ai/verify_v21_order_lifecycle_read_model.sh
}

run_v21_fill_execution_read_model() {
  echo "== verify_release: v0.21 fill/execution read model =="
  scripts/ai/verify_v21_fill_execution_read_model.sh
}

run_v21_risk_state_projection() {
  echo "== verify_release: v0.21 risk state projection =="
  scripts/ai/verify_v21_risk_state_projection.sh
}

run_v21_trader_terminal_readonly_dashboard() {
  echo "== verify_release: v0.21 Trader Terminal read-only dashboard foundation =="
  scripts/ai/verify_v21_trader_terminal_readonly_dashboard.sh
}

run_v21_release_gates() {
  echo "== verify_release: v0.21 release gates =="
  scripts/ai/verify_v21_release_gates.sh
}

run_v211_health_status_semantics() {
  echo "== verify_release: v0.21.1 health status semantics =="
  scripts/ai/verify_v211_health_status_semantics.sh
}

run_v211_read_model_projection_replay() {
  echo "== verify_release: v0.21.1 read-model projection replay =="
  scripts/ai/verify_v21_1_read_model_projection_replay.sh
}

run_v211_read_model_schema_boundary() {
  echo "== verify_release: v0.21.1 read-model JSON schema boundary =="
  scripts/ai/verify_v21_1_read_model_schema_boundary.sh
}

run_v211_trader_terminal_read_model_bridge() {
  echo "== verify_release: v0.21.1 Trader Terminal read-model runtime bridge =="
  scripts/ai/verify_v21_1_trader_terminal_read_model_bridge.sh
}

run_v211_release_gates() {
  echo "== verify_release: v0.21.1 release gates =="
  scripts/ai/verify_v21_1_release_gates.sh
}

run_v211_strict_provenance() {
  echo "== verify_release: v0.21.1 strict release provenance =="
  scripts/ai/verify_v21_1_strict_provenance.sh
}

run_v22_runtime_boundary_tests() {
  echo "== verify_release: v0.22 runtime degradation and boundary tests =="
  scripts/ai/verify_v22_runtime_boundary_tests.sh
}

run_v22_release_gates() {
  echo "== verify_release: v0.22 release gates =="
  scripts/ai/verify_v22_release_gates.sh
}

run_v22_strict_provenance() {
  echo "== verify_release: v0.22 strict release provenance =="
  scripts/ai/verify_release_strict.sh v22
}

run_v221_release_gates() {
  echo "== verify_release: v0.22.1 release gates =="
  scripts/ai/verify_v22_1_release_gates.sh
}

run_v221_strict_provenance() {
  echo "== verify_release: v0.22.1 strict release provenance =="
  scripts/ai/verify_v22_1_strict_provenance.sh
}

run_v23_release_gates() {
  echo "== verify_release: v0.23 release gates =="
  scripts/ai/verify_v23_release_gates.sh
}

run_v23_strict_provenance() {
  echo "== verify_release: v0.23 strict release provenance =="
  scripts/ai/verify_v23_strict_provenance.sh
}

run_v231_release_closeout_evidence() {
  echo "== verify_release: v0.23.1 release closeout evidence =="
  scripts/ai/verify_v23_1_release_closeout_evidence.sh
}

run_v231_stale_provenance_cleanup() {
  echo "== verify_release: v0.23.1 stale provenance cleanup =="
  scripts/ai/verify_v23_1_stale_provenance_cleanup.sh
}

run_v231_gate_phase_split() {
  echo "== verify_release: v0.23.1 gate phase split =="
  scripts/ai/verify_v23_1_gate_phase_split.sh
}

run_v231_evidence_replay_only_boundary() {
  echo "== verify_release: v0.23.1 evidence replay only boundary =="
  scripts/ai/verify_v23_1_evidence_replay_only_boundary.sh
}

run_v231_publication_evidence_audit_path() {
  echo "== verify_release: v0.23.1 publication evidence audit path =="
  scripts/ai/verify_v23_1_publication_evidence_audit_path.sh
}

run_v231_release_gates() {
  echo "== verify_release: v0.23.1 release gates =="
  scripts/ai/verify_v23_1_release_gates.sh
}

run_v231_strict_provenance() {
  echo "== verify_release: v0.23.1 strict release provenance =="
  scripts/ai/verify_v23_1_strict_provenance.sh
}

run_v24_intake_gate() {
  echo "== verify_release: v0.24.0 intake gate =="
  scripts/ai/verify_v24_intake_gate.sh
}

run_v24_order_control_contract() {
  echo "== verify_release: v0.24.0 order-control contract =="
  scripts/ai/verify_v24_order_control_contract.sh
}

run_v24_order_intent_policy() {
  echo "== verify_release: v0.24.0 order intent and execution policy model =="
  scripts/ai/verify_v24_order_intent_policy.sh
}

run_v24_rate_limit_throttle_gate() {
  echo "== verify_release: v0.24.0 rate-limit and throttle gate preview =="
  scripts/ai/verify_v24_rate_limit_throttle_gate.sh
}

run_v24_order_slicing_preview() {
  echo "== verify_release: v0.24.0 order slicing preview foundation =="
  scripts/ai/verify_v24_order_slicing_preview.sh
}

run_v24_cancel_replace_amend_preview() {
  echo "== verify_release: v0.24.0 cancel replace amend preview contract =="
  scripts/ai/verify_v24_cancel_replace_amend_preview.sh
}

run_v24_retry_policy_ledger() {
  echo "== verify_release: v0.24.0 retry no-retry policy ledger =="
  scripts/ai/verify_v24_retry_policy_ledger.sh
}

run_v24_readback_audit_evidence() {
  echo "== verify_release: v0.24.0 readback and audit evidence =="
  scripts/ai/verify_v24_readback_audit_evidence.sh
}

run_v24_dashboard_workbench_preview() {
  echo "== verify_release: v0.24.0 Dashboard Workbench order-control preview =="
  scripts/ai/verify_v24_dashboard_workbench_order_control_preview.sh
}

run_v24_release_gates() {
  echo "== verify_release: v0.24.0 release gates =="
  scripts/ai/verify_v24_release_gates.sh
}

run_v24_strict_provenance() {
  echo "== verify_release: v0.24.0 strict release provenance =="
  scripts/ai/verify_v24_strict_provenance.sh
}

run_v241_release_closeout_evidence() {
  echo "== verify_release: v0.24.1 release closeout evidence =="
  scripts/ai/verify_v24_1_release_closeout_evidence.sh
}

run_v241_provenance_reconciliation() {
  echo "== verify_release: v0.24.1 tag/main/release-body provenance reconciliation =="
  scripts/ai/verify_v24_1_provenance_reconciliation.sh
}

run_v241_stale_pretag_cleanup() {
  echo "== verify_release: v0.24.1 stale pre-tag evidence cleanup =="
  scripts/ai/verify_v24_1_stale_pretag_cleanup.sh
}

run_v241_schema_replay_classification() {
  echo "== verify_release: v0.24.1 schema replay classification =="
  scripts/ai/verify_v24_1_schema_replay_classification.sh
}

run_v241_dashboard_artifact_ingestion() {
  echo "== verify_release: v0.24.1 Dashboard artifact ingestion hardening =="
  scripts/ai/verify_v24_1_dashboard_artifact_ingestion.sh
}

run_v241_dashboard_fixture_ref_integrity() {
  echo "== verify_release: v0.24.1 Dashboard fixture ref integrity =="
  scripts/ai/verify_v24_1_dashboard_fixture_ref_integrity.sh
}

run_v241_release_gates() {
  echo "== verify_release: v0.24.1 release gates =="
  scripts/ai/verify_v24_1_release_gates.sh
}

run_v241_strict_provenance() {
  echo "== verify_release: v0.24.1 strict release provenance =="
  scripts/ai/verify_v24_1_strict_provenance.sh
}

run_v251_release_closeout_evidence() {
  echo "== verify_release: v0.25.1 release closeout evidence =="
  scripts/ai/verify_v25_1_release_closeout_evidence.sh
}

run_v251_corrective_release_scope() {
  echo "== verify_release: v0.25.1 corrective release scope =="
  scripts/ai/verify_v25_1_corrective_release_scope.sh
}

run_v251_stale_pretag_cleanup() {
  echo "== verify_release: v0.25.1 stale pre-tag cleanup =="
  scripts/ai/verify_v25_1_stale_pretag_cleanup.sh
}

run_v251_dashboard_source_ref_integrity() {
  echo "== verify_release: v0.25.1 Dashboard source_ref integrity =="
  scripts/ai/verify_v25_dashboard_monitoring_surface.sh
}

run_v251_post_release_gate_split() {
  echo "== verify_release: v0.25.1 post-release gate split =="
  scripts/ai/verify_v25_1_post_release_gate_split.sh
}

run_v251_release_gates() {
  echo "== verify_release: v0.25.1 release gates =="
  scripts/ai/verify_v25_1_release_gates.sh
}

run_v251_strict_provenance() {
  echo "== verify_release: v0.25.1 strict provenance =="
  scripts/ai/verify_v25_1_strict_provenance.sh
}

run_v26_intake_gate() {
  echo "== verify_release: v0.26.0 intake gate =="
  scripts/ai/verify_v26_intake_gate.sh
}

run_v26_product_hardening_boundary_contract() {
  echo "== verify_release: v0.26.0 product hardening boundary contract =="
  scripts/ai/verify_v26_product_hardening_boundary_contract.sh
}

run_v26_operator_permission_model() {
  echo "== verify_release: v0.26.0 operator permission model =="
  scripts/ai/verify_v26_operator_permission_model.sh
}

run_v26_operation_audit_trail() {
  echo "== verify_release: v0.26.0 operation audit trail =="
  scripts/ai/verify_v26_operation_audit_trail.sh
}

run_v26_deployment_provenance_model() {
  echo "== verify_release: v0.26.0 deployment provenance model =="
  scripts/ai/verify_v26_deployment_provenance_model.sh
}

run_v26_upgrade_rollback_runbook_evidence() {
  echo "== verify_release: v0.26.0 upgrade rollback runbook evidence =="
  scripts/ai/verify_v26_upgrade_rollback_runbook_evidence.sh
}

run_v26_slo_runbook_stability_evidence() {
  echo "== verify_release: v0.26.0 SLO runbook stability evidence =="
  scripts/ai/verify_v26_slo_runbook_stability_evidence.sh
}

run_v26_dashboard_admin_boundary_surface() {
  echo "== verify_release: v0.26.0 Dashboard admin boundary surface =="
  scripts/ai/verify_v26_dashboard_admin_boundary_surface.sh
}

run_v26_release_gates() {
  echo "== verify_release: v0.26.0 release gates =="
  scripts/ai/verify_v26_release_gates.sh
}

run_v26_strict_provenance() {
  echo "== verify_release: v0.26.0 strict provenance =="
  scripts/ai/verify_v26_strict_provenance.sh
}

run_v261_release_gates() {
  echo "== verify_release: v0.26.1 release gates =="
  scripts/ai/verify_v26_1_release_gates.sh
}

run_v261_strict_provenance() {
  echo "== verify_release: v0.26.1 strict provenance =="
  scripts/ai/verify_v26_1_strict_provenance.sh
}

run_v27_intake_gate() {
  echo "== verify_release: v0.27.0 intake gate =="
  scripts/ai/verify_v27_intake_gate.sh
}

run_v27_product_operations_boundary_contract() {
  echo "== verify_release: v0.27.0 product operations runtime integration boundary contract =="
  scripts/ai/verify_v27_product_operations_runtime_integration_boundary_contract.sh
}

run_v27_external_identity_permission_foundation() {
  echo "== verify_release: v0.27.0 external identity and permission integration foundation =="
  scripts/ai/verify_v27_external_identity_permission_foundation.sh
}

run_v27_persistent_audit_storage_foundation() {
  echo "== verify_release: v0.27.0 persistent operation audit storage foundation =="
  scripts/ai/verify_v27_persistent_audit_storage_foundation.sh
}

run_v27_deployment_orchestration_foundation() {
  echo "== verify_release: v0.27.0 deployment upgrade rollback orchestration foundation =="
  scripts/ai/verify_v27_deployment_orchestration_foundation.sh
}

run_v27_long_run_telemetry_slo_runtime_evidence() {
  echo "== verify_release: v0.27.0 long-run telemetry SLO runtime evidence =="
  scripts/ai/verify_v27_long_run_telemetry_slo_runtime_evidence.sh
}

run_v27_admin_workbench_runtime_state_bridge() {
  echo "== verify_release: v0.27.0 Admin Workbench runtime state bridge =="
  scripts/ai/verify_v27_admin_workbench_runtime_state_bridge.sh
}

run_v27_runtime_integration_fail_closed_hardening() {
  echo "== verify_release: v0.27.0 runtime integration fail-closed hardening =="
  scripts/ai/verify_v27_runtime_integration_fail_closed_hardening.sh
}

run_v27_release_gates() {
  echo "== verify_release: v0.27.0 release gates =="
  scripts/ai/verify_v27_release_gates.sh
}

run_v27_strict_provenance() {
  echo "== verify_release: v0.27.0 strict provenance =="
  scripts/ai/verify_v27_strict_provenance.sh
}

run_v271_release_gates() {
  echo "== verify_release: v0.27.1 release gates =="
  scripts/ai/verify_v27_1_release_gates.sh
}

run_v271_strict_provenance() {
  echo "== verify_release: v0.27.1 strict provenance =="
  scripts/ai/verify_v27_1_strict_provenance.sh
}

run_v28_intake_gate() {
  echo "== verify_release: v0.28.0 intake gate =="
  scripts/ai/verify_v28_intake_gate.sh
}

run_v28_backend_closure_boundary_contract() {
  echo "== verify_release: v0.28.0 backend closure boundary contract =="
  scripts/ai/verify_v28_backend_closure_boundary_contract.sh
}

run_v28_identity_permission_runtime_closure() {
  echo "== verify_release: v0.28.0 identity and permission runtime closure =="
  scripts/ai/verify_v28_identity_permission_runtime_closure.sh
}

run_v28_persistent_audit_storage_runtime_closure() {
  echo "== verify_release: v0.28.0 persistent audit storage runtime closure =="
  scripts/ai/verify_v28_persistent_audit_storage_runtime_closure.sh
}

run_v28_deployment_orchestration_runtime_closure() {
  echo "== verify_release: v0.28.0 deployment orchestration runtime closure =="
  scripts/ai/verify_v28_deployment_orchestration_runtime_closure.sh
}

run_v28_telemetry_slo_ingestion_runtime_closure() {
  echo "== verify_release: v0.28.0 telemetry SLO ingestion runtime closure =="
  scripts/ai/verify_v28_telemetry_slo_ingestion_runtime_closure.sh
}

run_v28_admin_workbench_backend_state_bridge_closure() {
  echo "== verify_release: v0.28.0 Admin Workbench backend state bridge closure =="
  scripts/ai/verify_v28_admin_workbench_backend_state_bridge_closure.sh
}

run_v28_trader_terminal_backend_api_contract_handoff() {
  echo "== verify_release: v0.28.0 Trader Terminal backend API contract handoff =="
  scripts/ai/verify_v28_trader_terminal_backend_api_contract_handoff.sh
}

run_v28_backend_closure_fail_closed_hardening() {
  echo "== verify_release: v0.28.0 backend closure fail-closed hardening =="
  scripts/ai/verify_v28_backend_closure_fail_closed_hardening.sh
}

run_v28_release_gates() {
  echo "== verify_release: v0.28.0 release gates =="
  scripts/ai/verify_v28_release_gates.sh
}

run_v28_strict_provenance() {
  echo "== verify_release: v0.28.0 strict provenance =="
  scripts/ai/verify_v28_strict_provenance.sh
}

run_v281_release_body_hash_normalization() {
  echo "== verify_release: v0.28.1 release body hash normalization =="
  scripts/ai/verify_v28_1_release_body_hash_normalization.sh
}

run_v281_runtime_closed_terminology() {
  echo "== verify_release: v0.28.1 runtime-closed terminology =="
  scripts/ai/verify_v28_1_runtime_closed_terminology.sh
}

run_v281_release_publish_after_gate_current_binding() {
  echo "== verify_release: v0.28.1 release publish-after-gate current binding =="
  scripts/ai/verify_release_publish_after_gate.sh
}

run_v281_release_gates() {
  echo "== verify_release: v0.28.1 release gates =="
  scripts/ai/verify_v28_1_release_gates.sh
}

run_v281_strict_provenance() {
  echo "== verify_release: v0.28.1 strict provenance =="
  scripts/ai/verify_v28_1_strict_provenance.sh
}

run_v29_intake_gate() {
  echo "== verify_release: v0.29.0 intake gate =="
  scripts/ai/verify_v29_intake_gate.sh
}

run_v29_backend_production_readiness_boundary_contract() {
  echo "== verify_release: v0.29.0 backend production readiness boundary contract =="
  scripts/ai/verify_v29_backend_production_readiness_boundary_contract.sh
}

run_v29_persistent_audit_storage_production_readiness() {
  echo "== verify_release: v0.29.0 persistent audit storage production readiness =="
  scripts/ai/verify_v29_persistent_audit_storage_production_readiness.sh
}

run_v29_telemetry_slo_ingestion_production_readiness() {
  echo "== verify_release: v0.29.0 telemetry SLO ingestion production readiness =="
  scripts/ai/verify_v29_telemetry_slo_ingestion_production_readiness.sh
}

run_v29_permission_source_production_readiness() {
  echo "== verify_release: v0.29.0 permission source production readiness =="
  scripts/ai/verify_v29_permission_source_production_readiness.sh
}

run_v29_read_only_backend_api_production_readiness() {
  echo "== verify_release: v0.29.0 read-only backend API production readiness =="
  scripts/ai/verify_v29_read_only_backend_api_production_readiness.sh
}

run_v29_deployment_config_runbook_production_readiness() {
  echo "== verify_release: v0.29.0 deployment config runbook production readiness =="
  scripts/ai/verify_v29_deployment_config_runbook_production_readiness.sh
}

run_v29_monitoring_alert_incident_production_readiness() {
  echo "== verify_release: v0.29.0 monitoring alert incident production readiness =="
  scripts/ai/verify_v29_monitoring_alert_incident_production_readiness.sh
}

run_v29_canary_rollback_dr_preflight_readiness() {
  echo "== verify_release: v0.29.0 canary rollback DR preflight readiness =="
  scripts/ai/verify_v29_canary_rollback_dr_preflight_readiness.sh
}

run_v29_backend_production_readiness_fail_closed_hardening() {
  echo "== verify_release: v0.29.0 backend production readiness fail-closed hardening =="
  scripts/ai/verify_v29_backend_production_readiness_fail_closed_hardening.sh
}

run_v29_release_gates() {
  echo "== verify_release: v0.29.0 release gates =="
  scripts/ai/verify_v29_release_gates.sh
}

run_v29_strict_provenance() {
  echo "== verify_release: v0.29.0 strict provenance =="
  scripts/ai/verify_v29_strict_provenance.sh
}

run_v30_intake_gate() {
  echo "== verify_release: v0.30.0 intake gate =="
  scripts/ai/verify_v30_intake_gate.sh
}

run_v30_backend_go_live_candidate_boundary_contract() {
  echo "== verify_release: v0.30.0 backend go-live candidate boundary contract =="
  scripts/ai/verify_v30_backend_go_live_candidate_boundary_contract.sh
}

run_v25_intake_gate() {
  echo "== verify_release: v0.25.0 intake gate =="
  scripts/ai/verify_v25_intake_gate.sh
}

run_v25_monitoring_observability_contract() {
  echo "== verify_release: v0.25.0 monitoring observability contract =="
  scripts/ai/verify_v25_monitoring_observability_contract.sh
}

run_v25_alert_taxonomy_routing() {
  echo "== verify_release: v0.25.0 alert taxonomy routing =="
  scripts/ai/verify_v25_alert_taxonomy_routing.sh
}

run_v25_incident_lifecycle_acknowledgement() {
  echo "== verify_release: v0.25.0 incident lifecycle acknowledgement =="
  scripts/ai/verify_v25_incident_lifecycle_acknowledgement.sh
}

run_v25_runbook_audit_evidence() {
  echo "== verify_release: v0.25.0 runbook audit evidence =="
  scripts/ai/verify_v25_runbook_audit_evidence.sh
}

run_v25_dr_preview_drill_evidence() {
  echo "== verify_release: v0.25.0 DR preview drill evidence =="
  scripts/ai/verify_v25_dr_preview_drill_evidence.sh
}

run_v25_dashboard_monitoring_surface() {
  echo "== verify_release: v0.25.0 Dashboard monitoring surface =="
  scripts/ai/verify_v25_dashboard_monitoring_surface.sh
}

run_v25_slo_freshness_diagnostics_gate() {
  echo "== verify_release: v0.25.0 SLO freshness diagnostics gate =="
  scripts/ai/verify_v25_slo_freshness_diagnostics_gate.sh
}

run_v25_release_gates() {
  echo "== verify_release: v0.25.0 release gates =="
  scripts/ai/verify_v25_release_gates.sh
}

run_v25_strict_provenance() {
  echo "== verify_release: v0.25.0 strict provenance =="
  scripts/ai/verify_v25_strict_provenance.sh
}

run_v171_release_hardening() {
  echo "== verify_release: v0.17.1 release hardening =="
  scripts/ai/verify_v171_release_hardening.sh
}

run_v18_strict_provenance() {
  echo "== verify_release: v0.18 strict binary provenance =="
  scripts/ai/verify_release_strict.sh v18
}

run_v19_strict_provenance() {
  echo "== verify_release: v0.19 strict release provenance =="
  scripts/ai/verify_release_strict.sh v19
}

run_v20_strict_provenance() {
  echo "== verify_release: v0.20 strict release provenance =="
  scripts/ai/verify_release_strict.sh v20
}

run_v21_strict_provenance() {
  echo "== verify_release: v0.21 strict release provenance =="
  scripts/ai/verify_release_strict.sh v21
}

run_release_surface_current_guard() {
  echo "== verify_release: release surface current guard =="
  scripts/ai/check_release_surface_current.sh
}

run_release_publication_guard() {
  echo "== verify_release: release publication guard =="
  scripts/ai/check_github_release_published.sh
}

run_release_publish_after_gate() {
  echo "== verify_release: release publish after gate =="
  scripts/ai/verify_release_publish_after_gate.sh
}

run_stage() {
  local stage="$1"
  case "$stage" in
    all)
      run_full_checks
      run_release_build_product_surface
      run_rust_only_gates
      run_v02_supervisor_smoke
      run_v03_supervisor_control_smoke
      run_v03_dashboard_smoke
      run_v05_workflow_artifacts_smoke
      run_v06_binance_testnet_dry_run_smoke
      run_v07_default_offline_gate
      run_v07_manual_online_preflight
      run_v08_default_offline_gate
      run_v08_authenticated_readonly_preflight
      run_v09_strategy_runtime_smoke
      run_v09_shadow_mode_no_order_gate
      run_v091_strategy_supervisor_dashboard_integration
      run_v10_offline_release_gates
      run_v10_manual_order_proof_preflight
      run_v11_offline_release_gates
      run_v12_offline_release_gates
      run_v12_manual_online_preflight
      run_v13_no_production_mutation_gate
      run_v14_release_gates
      run_v15_release_gates
      run_v151_release_gates
      run_v16_release_gates
      run_v17_release_gates
      run_v18_release_gates
      run_v171_release_hardening
      run_v19_release_gates
      run_v19_strict_provenance
      run_v20_release_gates
      run_v20_strict_provenance
      run_v20_patch_release_gates
      run_v21_read_model_contract
      run_v21_account_snapshot_read_model
      run_v21_position_read_model
      run_v21_order_lifecycle_read_model
      run_v21_fill_execution_read_model
      run_v21_risk_state_projection
      run_v21_trader_terminal_readonly_dashboard
      run_v21_release_gates
      run_v21_strict_provenance
      run_v211_health_status_semantics
      run_v211_read_model_projection_replay
      run_v211_read_model_schema_boundary
      run_v211_trader_terminal_read_model_bridge
      run_v211_release_gates
      run_v211_strict_provenance
      run_v22_runtime_boundary_tests
      run_v22_release_gates
      run_v22_strict_provenance
      run_v221_release_gates
      run_v221_strict_provenance
      run_v23_release_gates
      run_v23_strict_provenance
      run_v231_release_closeout_evidence
      run_v231_stale_provenance_cleanup
      run_v231_gate_phase_split
      run_v231_evidence_replay_only_boundary
      run_v231_publication_evidence_audit_path
      run_v231_release_gates
      run_v231_strict_provenance
      run_v24_intake_gate
      run_v24_order_control_contract
      run_v24_order_intent_policy
      run_v24_rate_limit_throttle_gate
      run_v24_order_slicing_preview
      run_v24_cancel_replace_amend_preview
      run_v24_retry_policy_ledger
      run_v24_readback_audit_evidence
      run_v24_dashboard_workbench_preview
      run_v24_release_gates
      run_v24_strict_provenance
      run_v241_release_closeout_evidence
      run_v241_provenance_reconciliation
      run_v241_stale_pretag_cleanup
      run_v241_schema_replay_classification
      run_v241_dashboard_artifact_ingestion
      run_v241_dashboard_fixture_ref_integrity
      run_v241_release_gates
      run_v241_strict_provenance
      run_v25_intake_gate
      run_v25_monitoring_observability_contract
      run_v25_alert_taxonomy_routing
      run_v25_incident_lifecycle_acknowledgement
      run_v25_runbook_audit_evidence
      run_v25_dr_preview_drill_evidence
      run_v25_dashboard_monitoring_surface
      run_v25_slo_freshness_diagnostics_gate
      run_v25_release_gates
      run_v25_strict_provenance
      run_v251_release_closeout_evidence
      run_v251_corrective_release_scope
      run_v251_stale_pretag_cleanup
      run_v251_dashboard_source_ref_integrity
      run_v251_post_release_gate_split
      run_v251_release_gates
      run_v251_strict_provenance
      run_v26_intake_gate
      run_v26_product_hardening_boundary_contract
      run_v26_operator_permission_model
      run_v26_operation_audit_trail
      run_v26_deployment_provenance_model
      run_v26_upgrade_rollback_runbook_evidence
      run_v26_slo_runbook_stability_evidence
      run_v26_dashboard_admin_boundary_surface
      run_v26_release_gates
      run_v26_strict_provenance
      run_v261_release_gates
      run_v261_strict_provenance
      run_release_surface_current_guard
      run_release_publication_guard
      run_release_publish_after_gate
      ;;
    full)
      run_full_checks
      ;;
    release-build-product-surface)
      run_release_build_product_surface
      ;;
    rust-only-gates)
      run_rust_only_gates
      ;;
    v02-supervisor-smoke)
      run_v02_supervisor_smoke
      ;;
    v03-supervisor-control-smoke)
      run_v03_supervisor_control_smoke
      ;;
    v03-dashboard-smoke)
      run_v03_dashboard_smoke
      ;;
    v05-workflow-artifacts-smoke)
      run_v05_workflow_artifacts_smoke
      ;;
    v06-binance-testnet-dry-run-smoke)
      run_v06_binance_testnet_dry_run_smoke
      ;;
    v07-default-offline-gate)
      run_v07_default_offline_gate
      ;;
    v07-manual-online-preflight)
      run_v07_manual_online_preflight
      ;;
    v08-default-offline-gate)
      run_v08_default_offline_gate
      ;;
    v08-authenticated-readonly-preflight)
      run_v08_authenticated_readonly_preflight
      ;;
    v09-strategy-runtime-smoke)
      run_v09_strategy_runtime_smoke
      ;;
    v09-shadow-mode-no-order-gate)
      run_v09_shadow_mode_no_order_gate
      ;;
    v091-strategy-supervisor-dashboard-integration)
      run_v091_strategy_supervisor_dashboard_integration
      ;;
    v10-offline-release-gates)
      run_v10_offline_release_gates
      ;;
    v10-manual-order-proof-preflight)
      run_v10_manual_order_proof_preflight
      ;;
    v11-offline-release-gates)
      run_v11_offline_release_gates
      ;;
    v12-offline-release-gates)
      run_v12_offline_release_gates
      ;;
    v12-manual-online-preflight)
      run_v12_manual_online_preflight
      ;;
    v13-no-production-mutation-gate)
      run_v13_no_production_mutation_gate
      ;;
    v14-release-gates)
      run_v14_release_gates
      ;;
    v15-release-gates)
      run_v15_release_gates
      ;;
    v151-release-gates)
      run_v151_release_gates
      ;;
    v16-release-gates)
      run_v16_release_gates
      ;;
    v17-release-gates)
      run_v17_release_gates
      ;;
    v18-release-gates)
      run_v18_release_gates
      ;;
    v19-release-gates)
      run_v19_release_gates
      ;;
    v20-release-gates)
      run_v20_release_gates
      ;;
    v20.1-release-gates)
      run_v20_patch_release_gates
      ;;
    v21-read-model-contract)
      run_v21_read_model_contract
      ;;
    v21-account-snapshot-read-model)
      run_v21_account_snapshot_read_model
      ;;
    v21-position-read-model)
      run_v21_position_read_model
      ;;
    v21-order-lifecycle-read-model)
      run_v21_order_lifecycle_read_model
      ;;
    v21-fill-execution-read-model)
      run_v21_fill_execution_read_model
      ;;
    v21-risk-state-projection)
      run_v21_risk_state_projection
      ;;
    v21-trader-terminal-readonly-dashboard)
      run_v21_trader_terminal_readonly_dashboard
      ;;
    v21-release-gates)
      run_v21_release_gates
      ;;
    v21-strict-provenance)
      run_v21_strict_provenance
      ;;
    v21.1-health-status-semantics)
      run_v211_health_status_semantics
      ;;
    v21.1-read-model-projection-replay)
      run_v211_read_model_projection_replay
      ;;
    v21.1-read-model-schema-boundary)
      run_v211_read_model_schema_boundary
      ;;
    v21.1-trader-terminal-read-model-bridge)
      run_v211_trader_terminal_read_model_bridge
      ;;
    v21.1-release-gates)
      run_v211_release_gates
      ;;
    v21.1-strict-provenance)
      run_v211_strict_provenance
      ;;
    v22-runtime-boundary-tests)
      run_v22_runtime_boundary_tests
      ;;
    v22-release-gates)
      run_v22_release_gates
      ;;
    v22-strict-provenance)
      run_v22_strict_provenance
      ;;
    v22.1-release-gates)
      run_v221_release_gates
      ;;
    v22.1-strict-provenance)
      run_v221_strict_provenance
      ;;
    v23-release-gates)
      run_v23_release_gates
      ;;
    v23-strict-provenance)
      run_v23_strict_provenance
      ;;
    v23.1-release-closeout-evidence)
      run_v231_release_closeout_evidence
      ;;
    v23.1-stale-provenance-cleanup)
      run_v231_stale_provenance_cleanup
      ;;
    v23.1-gate-phase-split)
      run_v231_gate_phase_split
      ;;
    v23.1-evidence-replay-only-boundary)
      run_v231_evidence_replay_only_boundary
      ;;
    v23.1-publication-evidence-audit-path)
      run_v231_publication_evidence_audit_path
      ;;
    v23.1-release-gates)
      run_v231_release_gates
      ;;
    v23.1-strict-provenance)
      run_v231_strict_provenance
      ;;
    v24-intake-gate)
      run_v24_intake_gate
      ;;
    v24-order-control-contract)
      run_v24_order_control_contract
      ;;
    v24-order-intent-policy)
      run_v24_order_intent_policy
      ;;
    v24-rate-limit-throttle-gate)
      run_v24_rate_limit_throttle_gate
      ;;
    v24-order-slicing-preview)
      run_v24_order_slicing_preview
      ;;
    v24-cancel-replace-amend-preview)
      run_v24_cancel_replace_amend_preview
      ;;
    v24-retry-policy-ledger)
      run_v24_retry_policy_ledger
      ;;
    v24-readback-audit-evidence)
      run_v24_readback_audit_evidence
      ;;
    v24-dashboard-workbench-preview)
      run_v24_dashboard_workbench_preview
      ;;
    v24-release-gates)
      run_v24_release_gates
      ;;
    v24-strict-provenance)
      run_v24_strict_provenance
      ;;
    v24.1-release-closeout-evidence)
      run_v241_release_closeout_evidence
      ;;
    v24.1-provenance-reconciliation)
      run_v241_provenance_reconciliation
      ;;
    v24.1-stale-pretag-cleanup)
      run_v241_stale_pretag_cleanup
      ;;
    v24.1-schema-replay-classification)
      run_v241_schema_replay_classification
      ;;
    v24.1-dashboard-artifact-ingestion)
      run_v241_dashboard_artifact_ingestion
      ;;
    v24.1-dashboard-fixture-ref-integrity)
      run_v241_dashboard_fixture_ref_integrity
      ;;
    v24.1-release-gates)
      run_v241_release_gates
      ;;
    v24.1-strict-provenance)
      run_v241_strict_provenance
      ;;
    v25.1-release-closeout-evidence)
      run_v251_release_closeout_evidence
      ;;
    v25.1-corrective-release-scope)
      run_v251_corrective_release_scope
      ;;
    v25.1-stale-pretag-cleanup)
      run_v251_stale_pretag_cleanup
      ;;
    v25.1-dashboard-source-ref-integrity)
      run_v251_dashboard_source_ref_integrity
      ;;
    v25.1-post-release-gate-split)
      run_v251_post_release_gate_split
      ;;
    v25.1-release-gates)
      run_v251_release_gates
      ;;
    v25.1-strict-provenance)
      run_v251_strict_provenance
      ;;
    v26-intake-gate)
      run_v26_intake_gate
      ;;
    v26-product-hardening-boundary-contract)
      run_v26_product_hardening_boundary_contract
      ;;
    v26-operator-permission-model)
      run_v26_operator_permission_model
      ;;
    v26-operation-audit-trail)
      run_v26_operation_audit_trail
      ;;
    v26-deployment-provenance-model)
      run_v26_deployment_provenance_model
      ;;
    v26-upgrade-rollback-runbook-evidence)
      run_v26_upgrade_rollback_runbook_evidence
      ;;
    v26-slo-runbook-stability-evidence)
      run_v26_slo_runbook_stability_evidence
      ;;
    v26-dashboard-admin-boundary-surface)
      run_v26_dashboard_admin_boundary_surface
      ;;
    v26-release-gates)
      run_v26_release_gates
      ;;
    v26-strict-provenance)
      run_v26_strict_provenance
      ;;
    v26.1-release-gates)
      run_v261_release_gates
      ;;
    v26.1-strict-provenance)
      run_v261_strict_provenance
      ;;
    v27-intake-gate)
      run_v27_intake_gate
      ;;
    v27-product-operations-boundary-contract)
      run_v27_product_operations_boundary_contract
      ;;
    v27-external-identity-permission-foundation)
      run_v27_external_identity_permission_foundation
      ;;
    v27-persistent-audit-storage-foundation)
      run_v27_persistent_audit_storage_foundation
      ;;
    v27-deployment-orchestration-foundation)
      run_v27_deployment_orchestration_foundation
      ;;
    v27-long-run-telemetry-slo-runtime-evidence)
      run_v27_long_run_telemetry_slo_runtime_evidence
      ;;
    v27-admin-workbench-runtime-state-bridge)
      run_v27_admin_workbench_runtime_state_bridge
      ;;
    v27-runtime-integration-fail-closed-hardening)
      run_v27_runtime_integration_fail_closed_hardening
      ;;
    v27-release-gates)
      run_v27_release_gates
      ;;
    v27-strict-provenance)
      run_v27_strict_provenance
      ;;
    v27.1-release-gates)
      run_v271_release_gates
      ;;
    v27.1-strict-provenance)
      run_v271_strict_provenance
      ;;
    v28-intake-gate)
      run_v28_intake_gate
      ;;
    v28-backend-closure-boundary-contract)
      run_v28_backend_closure_boundary_contract
      ;;
    v28-identity-permission-runtime-closure)
      run_v28_identity_permission_runtime_closure
      ;;
    v28-persistent-audit-storage-runtime-closure)
      run_v28_persistent_audit_storage_runtime_closure
      ;;
    v28-deployment-orchestration-runtime-closure)
      run_v28_deployment_orchestration_runtime_closure
      ;;
    v28-telemetry-slo-ingestion-runtime-closure)
      run_v28_telemetry_slo_ingestion_runtime_closure
      ;;
    v28-admin-workbench-backend-state-bridge-closure)
      run_v28_admin_workbench_backend_state_bridge_closure
      ;;
    v28-trader-terminal-backend-api-contract-handoff)
      run_v28_trader_terminal_backend_api_contract_handoff
      ;;
    v28-backend-closure-fail-closed-hardening)
      run_v28_backend_closure_fail_closed_hardening
      ;;
    v28-release-gates)
      run_v28_release_gates
      ;;
    v28-strict-provenance)
      run_v28_strict_provenance
      ;;
    v28.1-release-body-hash-normalization)
      run_v281_release_body_hash_normalization
      ;;
    v28.1-runtime-closed-terminology)
      run_v281_runtime_closed_terminology
      ;;
    v28.1-release-publish-after-gate-current-binding)
      run_v281_release_publish_after_gate_current_binding
      ;;
    v28.1-release-gates)
      run_v281_release_gates
      ;;
    v28.1-strict-provenance)
      run_v281_strict_provenance
      ;;
    v29-intake-gate)
      run_v29_intake_gate
      ;;
    v29-backend-production-readiness-boundary-contract)
      run_v29_backend_production_readiness_boundary_contract
      ;;
    v29-persistent-audit-storage-production-readiness)
      run_v29_persistent_audit_storage_production_readiness
      ;;
    v29-telemetry-slo-ingestion-production-readiness)
      run_v29_telemetry_slo_ingestion_production_readiness
      ;;
    v29-permission-source-production-readiness)
      run_v29_permission_source_production_readiness
      ;;
    v29-read-only-backend-api-production-readiness)
      run_v29_read_only_backend_api_production_readiness
      ;;
    v29-deployment-config-runbook-production-readiness)
      run_v29_deployment_config_runbook_production_readiness
      ;;
    v29-monitoring-alert-incident-production-readiness)
      run_v29_monitoring_alert_incident_production_readiness
      ;;
    v29-canary-rollback-dr-preflight-readiness)
      run_v29_canary_rollback_dr_preflight_readiness
      ;;
    v29-backend-production-readiness-fail-closed-hardening)
      run_v29_backend_production_readiness_fail_closed_hardening
      ;;
    v29-release-gates)
      run_v29_release_gates
      ;;
    v29-strict-provenance)
      run_v29_strict_provenance
      ;;
    v30-intake-gate)
      run_v30_intake_gate
      ;;
    v30-backend-go-live-candidate-boundary-contract)
      run_v30_backend_go_live_candidate_boundary_contract
      ;;
    v25-intake-gate)
      run_v25_intake_gate
      ;;
    v25-monitoring-observability-contract)
      run_v25_monitoring_observability_contract
      ;;
    v25-alert-taxonomy-routing)
      run_v25_alert_taxonomy_routing
      ;;
    v25-incident-lifecycle-acknowledgement)
      run_v25_incident_lifecycle_acknowledgement
      ;;
    v25-runbook-audit-evidence)
      run_v25_runbook_audit_evidence
      ;;
    v25-dr-preview-drill-evidence)
      run_v25_dr_preview_drill_evidence
      ;;
    v25-dashboard-monitoring-surface)
      run_v25_dashboard_monitoring_surface
      ;;
    v25-slo-freshness-diagnostics-gate)
      run_v25_slo_freshness_diagnostics_gate
      ;;
    v25-release-gates)
      run_v25_release_gates
      ;;
    v25-strict-provenance)
      run_v25_strict_provenance
      ;;
    v171-release-hardening)
      run_v171_release_hardening
      ;;
    v18-strict-provenance)
      run_v18_strict_provenance
      ;;
    v19-strict-provenance)
      run_v19_strict_provenance
      ;;
    v20-strict-provenance)
      run_v20_strict_provenance
      ;;
    release-surface-current-guard)
      run_release_surface_current_guard
      ;;
    release-publication-guard)
      run_release_publication_guard
      ;;
    release-publish-after-gate)
      run_release_publish_after_gate
      ;;
    *)
      echo "unknown verify_release stage: $stage" >&2
      echo "valid stages: all, full, release-build-product-surface, rust-only-gates, v02-supervisor-smoke, v03-supervisor-control-smoke, v03-dashboard-smoke, v05-workflow-artifacts-smoke, v06-binance-testnet-dry-run-smoke, v07-default-offline-gate, v07-manual-online-preflight, v08-default-offline-gate, v08-authenticated-readonly-preflight, v09-strategy-runtime-smoke, v09-shadow-mode-no-order-gate, v091-strategy-supervisor-dashboard-integration, v10-offline-release-gates, v10-manual-order-proof-preflight, v11-offline-release-gates, v12-offline-release-gates, v12-manual-online-preflight, v13-no-production-mutation-gate, v14-release-gates, v15-release-gates, v151-release-gates, v16-release-gates, v17-release-gates, v18-release-gates, v19-release-gates, v20-release-gates, v20.1-release-gates, v21-read-model-contract, v21-account-snapshot-read-model, v21-position-read-model, v21-order-lifecycle-read-model, v21-fill-execution-read-model, v21-risk-state-projection, v21-trader-terminal-readonly-dashboard, v21-release-gates, v21-strict-provenance, v21.1-health-status-semantics, v21.1-read-model-projection-replay, v21.1-read-model-schema-boundary, v21.1-trader-terminal-read-model-bridge, v21.1-release-gates, v21.1-strict-provenance, v22-runtime-boundary-tests, v22-release-gates, v22-strict-provenance, v22.1-release-gates, v22.1-strict-provenance, v23-release-gates, v23-strict-provenance, v23.1-release-closeout-evidence, v23.1-stale-provenance-cleanup, v23.1-gate-phase-split, v23.1-evidence-replay-only-boundary, v23.1-publication-evidence-audit-path, v23.1-release-gates, v23.1-strict-provenance, v24-intake-gate, v24-order-control-contract, v24-order-intent-policy, v24-rate-limit-throttle-gate, v24-order-slicing-preview, v24-cancel-replace-amend-preview, v24-retry-policy-ledger, v24-readback-audit-evidence, v24-dashboard-workbench-preview, v24-release-gates, v24-strict-provenance, v24.1-release-closeout-evidence, v24.1-provenance-reconciliation, v24.1-stale-pretag-cleanup, v24.1-schema-replay-classification, v24.1-dashboard-artifact-ingestion, v24.1-dashboard-fixture-ref-integrity, v24.1-release-gates, v24.1-strict-provenance, v25-intake-gate, v25-monitoring-observability-contract, v25-alert-taxonomy-routing, v25-incident-lifecycle-acknowledgement, v25-runbook-audit-evidence, v25-dr-preview-drill-evidence, v25-dashboard-monitoring-surface, v25-slo-freshness-diagnostics-gate, v25-release-gates, v25-strict-provenance, v25.1-release-closeout-evidence, v25.1-corrective-release-scope, v25.1-stale-pretag-cleanup, v25.1-dashboard-source-ref-integrity, v25.1-post-release-gate-split, v25.1-release-gates, v25.1-strict-provenance, v26-intake-gate, v26-product-hardening-boundary-contract, v26-operator-permission-model, v26-operation-audit-trail, v26-deployment-provenance-model, v26-upgrade-rollback-runbook-evidence, v26-slo-runbook-stability-evidence, v26-dashboard-admin-boundary-surface, v26-release-gates, v26-strict-provenance, v26.1-release-gates, v26.1-strict-provenance, v27-intake-gate, v27-product-operations-boundary-contract, v27-external-identity-permission-foundation, v27-persistent-audit-storage-foundation, v27-deployment-orchestration-foundation, v27-long-run-telemetry-slo-runtime-evidence, v27-admin-workbench-runtime-state-bridge, v27-runtime-integration-fail-closed-hardening, v27-release-gates, v27-strict-provenance, v27.1-release-gates, v27.1-strict-provenance, v28-intake-gate, v28-backend-closure-boundary-contract, v28-identity-permission-runtime-closure, v28-persistent-audit-storage-runtime-closure, v28-deployment-orchestration-runtime-closure, v28-telemetry-slo-ingestion-runtime-closure, v28-admin-workbench-backend-state-bridge-closure, v28-trader-terminal-backend-api-contract-handoff, v28-backend-closure-fail-closed-hardening, v28-release-gates, v28-strict-provenance, v28.1-release-body-hash-normalization, v28.1-runtime-closed-terminology, v28.1-release-publish-after-gate-current-binding, v30-intake-gate, v171-release-hardening, v18-strict-provenance, v19-strict-provenance, v20-strict-provenance, release-surface-current-guard, release-publication-guard, release-publish-after-gate" >&2
      echo "additional v28/v29/v30 stages: v28-trader-terminal-backend-api-contract-handoff, v28-backend-closure-fail-closed-hardening, v28-release-gates, v28-strict-provenance, v28.1-runtime-closed-terminology, v28.1-release-publish-after-gate-current-binding, v28.1-release-gates, v28.1-strict-provenance, v29-intake-gate, v29-backend-production-readiness-boundary-contract, v29-persistent-audit-storage-production-readiness, v29-telemetry-slo-ingestion-production-readiness, v29-permission-source-production-readiness, v29-read-only-backend-api-production-readiness, v29-deployment-config-runbook-production-readiness, v29-monitoring-alert-incident-production-readiness, v29-canary-rollback-dr-preflight-readiness, v29-backend-production-readiness-fail-closed-hardening, v29-release-gates, v29-strict-provenance, v30-intake-gate, v30-backend-go-live-candidate-boundary-contract" >&2
      exit 2
      ;;
  esac
}

if (( $# == 0 )); then
  run_stage all
else
  for stage in "$@"; do
    run_stage "$stage"
  done
fi

echo "== verify_release complete =="
