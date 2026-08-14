#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

FEATURES="${NAUTILUS_RUST_FEATURES:-arrow,ffi,high-precision,streaming,defi}"
NAUTILUS_RELEASE_BIN="$ROOT/target/release/nautilus"
NTPRO_NODE_RELEASE_BIN="$ROOT/target/release/ntpro-node"

fail() {
  echo "current release verification failed: $*" >&2
  exit 1
}

require_help_contains() {
  local file="$1"
  shift
  local pattern
  for pattern in "$@"; do
    grep -Eiq -- "$pattern" "$file" || fail "help output missing '$pattern': $file"
  done
}

release_cli_feature_args() {
  local selected=()
  local feature
  IFS=',' read -r -a feature_list <<<"$FEATURES"
  for feature in "${feature_list[@]}"; do
    feature="${feature//[[:space:]]/}"
    [[ "$feature" == "defi" ]] && selected+=("$feature")
  done
  if (( ${#selected[@]} > 0 )); then
    printf '%s\n' --features
    (IFS=','; printf '%s\n' "${selected[*]}")
  fi
}

ensure_release_cli_binaries() {
  local cli_feature_args=()
  while IFS= read -r arg; do
    cli_feature_args+=("$arg")
  done < <(release_cli_feature_args)

  cargo build --locked -p nautilus-cli --release \
    --bin nautilus --bin ntpro-node "${cli_feature_args[@]}"
  [[ -x "$NAUTILUS_RELEASE_BIN" ]] || fail "missing binary: $NAUTILUS_RELEASE_BIN"
  [[ -x "$NTPRO_NODE_RELEASE_BIN" ]] || fail "missing binary: $NTPRO_NODE_RELEASE_BIN"
}

run_release_build_product_surface() {
  ensure_release_cli_binaries
  cargo metadata --no-deps --format-version=1 | grep -q '"name":"nautilus-cli"' \
    || fail "nautilus-cli package is missing"

  "$NAUTILUS_RELEASE_BIN" --help >/tmp/nautilus_cli_help.txt
  require_help_contains /tmp/nautilus_cli_help.txt backtest live sandbox data supervisor
  "$NAUTILUS_RELEASE_BIN" supervisor --help >/tmp/nautilus_supervisor_help.txt
  require_help_contains /tmp/nautilus_supervisor_help.txt \
    register list start stop pause resume reconnect-data reconnect-execution status logs metrics sandbox-only
  "$NTPRO_NODE_RELEASE_BIN" --help >/tmp/ntpro_node_help.txt
  require_help_contains /tmp/ntpro_node_help.txt config run-id output stop-file sandbox-only
}

run_rust_only_gates() {
  scripts/ai/check_rust_only_runtime.sh
  scripts/ai/check_cython_removed.sh
}

run_current_governance() {
  scripts/ai/check_release_surface_current.sh
  scripts/ai/check_docs_examples_governance.sh
  scripts/ai/check_zero_python_closeout.sh
  scripts/ai/check_backend_hygiene.sh
  scripts/ai/check_ignored_tests_current_register.sh
  scripts/ai/test_ignored_tests_current_register.sh
  scripts/ai/check_backend_runtime_risk_inventory.sh
  scripts/ai/test_backend_runtime_risk_inventory.sh
  scripts/ai/check_control_plane_retired.sh
  scripts/ai/check_historical_release_retirement.sh
  scripts/ai/check_s3_live_closeout.sh
  scripts/ai/ntpro_governance.sh golden-trace \
    tests/golden/schema_smoke.jsonl --mode validate-only
}

run_backend_freeze_baseline() {
  scripts/ai/check_backend_freeze_baseline.sh
}

run_s3_live_closeout() {
  scripts/ai/check_s3_live_closeout.sh
}

run_s3_live_exit_evidence() {
  scripts/ai/check_s3_live_closeout.sh
  scripts/ai/test_s3_live_exit_evidence.sh
}

run_backend_performance_baseline() {
  scripts/ai/check_backend_performance_baseline.sh
}

run_backend_performance_hosted() {
  scripts/ai/check_backend_performance_hosted.sh
}

run_release_publication_guard() {
  scripts/ai/check_github_release_published.sh
}

run_release_publish_after_gate() {
  scripts/ai/verify_release_publish_after_gate.sh
}

run_v33_maintenance_release() {
  scripts/ai/check_backend_maintenance_release.sh
}

run_v33_strict_provenance() {
  scripts/ai/check_release_strict_provenance.sh
}

run_current_release_gates() {
  run_current_governance
  run_backend_freeze_baseline
  run_rust_only_gates
  run_v33_maintenance_release
  run_v33_strict_provenance
  run_release_publication_guard
  run_release_publish_after_gate
}

run_stage() {
  case "$1" in
    all|current-release-gates)
      run_current_release_gates
      ;;
    full)
      scripts/ai/verify_full.sh
      ;;
    release-build-product-surface)
      run_release_build_product_surface
      ;;
    rust-only-gates)
      run_rust_only_gates
      ;;
    current-governance)
      run_current_governance
      ;;
    s3-live-closeout)
      run_s3_live_closeout
      ;;
    s3-live-exit-evidence)
      run_s3_live_exit_evidence
      ;;
    backend-freeze-baseline)
      run_backend_freeze_baseline
      ;;
    backend-performance-baseline)
      run_backend_performance_baseline
      ;;
    backend-performance-hosted)
      run_backend_performance_hosted
      ;;
    release-surface-current-guard)
      scripts/ai/check_release_surface_current.sh
      ;;
    release-publication-guard)
      run_release_publication_guard
      ;;
    release-publish-after-gate)
      run_release_publish_after_gate
      ;;
    v33-maintenance-release)
      run_v33_maintenance_release
      ;;
    v33-strict-provenance)
      run_v33_strict_provenance
      ;;
    v*)
      fail "historical release stage retired by PTC-006: $1"
      ;;
    *)
      fail "unknown stage '$1'; valid stages: current-release-gates, full, release-build-product-surface, rust-only-gates, current-governance, s3-live-closeout, s3-live-exit-evidence, backend-freeze-baseline, backend-performance-baseline, backend-performance-hosted, v33-maintenance-release, v33-strict-provenance, release-surface-current-guard, release-publication-guard, release-publish-after-gate"
      ;;
  esac
}

if (( $# == 0 )); then
  run_stage current-release-gates
else
  for stage in "$@"; do
    run_stage "$stage"
  done
fi

echo "== verify_release complete =="
