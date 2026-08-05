#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <changed-files-list>" >&2
  exit 2
fi

changed_files_path="$1"
if [[ ! -f "$changed_files_path" ]]; then
  echo "changed-files list does not exist: $changed_files_path" >&2
  exit 2
fi

if [[ -z "${GITHUB_OUTPUT:-}" ]]; then
  echo "GITHUB_OUTPUT must be set" >&2
  exit 2
fi

matches() {
  grep -E "$1" "$changed_files_path" >/dev/null 2>&1
}

heavy_rust=false
release_verify=false
institution_workbench=false
strategy_workbench=false
control_center=false
mvp_acceptance=false
mvp_fault_matrix=false
mvp_final_acceptance=false
security_workflow=false
security_dependencies=false
frontend_app=false

if matches '^(Cargo\.(toml|lock)|crates/|tests/|examples/|configs/)'; then
  heavy_rust=true
fi

if matches '^apps/strategy-workbench/'; then
  frontend_app=true
fi

if matches '^(\.github/(workflows|actions)/|scripts/ai/|docs/rust-cutover/(governance|release|golden_trace)/|tests/golden/|crates/governance/|scripts/ci/(classify-ci-changes|security-audit-gate|test-ci-change-classifier)\.sh$)'; then
  release_verify=true
fi

if matches '^(crates/cli/src/dashboard/(institution_workbench\.rs|server\.rs|server/tests\.rs)|scripts/ai/test_institution_workbench_(contract|browser)\.mjs)'; then
  institution_workbench=true
fi

if matches '^(crates/cli/src/dashboard/(strategy_workbench\.rs|server\.rs|server/tests\.rs)|scripts/ai/test_strategy_workbench_(contract|browser)\.mjs)'; then
  strategy_workbench=true
fi

if matches '^(crates/cli/src/dashboard/(control_center\.rs|server\.rs|server/tests\.rs)|scripts/ai/test_control_center_(contract|browser)\.mjs)'; then
  control_center=true
fi

if matches '^(Cargo\.(toml|lock)|\.cargo/.*|crates/.*|scripts/ai/test_mvp_acceptance\.mjs|configs/nodes/btc-ema-shadow\.toml|examples/rust/backtest/minimal_engine_smoke\.toml|\.github/workflows/rust-cutover-smoke\.yml|scripts/ci/(classify-ci-changes|test-ci-change-classifier)\.sh)$'; then
  mvp_acceptance=true
fi

if matches '^(Cargo\.(toml|lock)|\.cargo/.*|crates/.*|scripts/ai/test_mvp_fault_matrix\.mjs|configs/nodes/btc-ema-shadow\.toml|\.github/workflows/rust-cutover-smoke\.yml|scripts/ci/(classify-ci-changes|test-ci-change-classifier)\.sh)$'; then
  mvp_fault_matrix=true
fi

freeze_source_match=false
if [[ -f docs/product/mvp_freeze_manifest.json ]]; then
  freeze_sources="$(mktemp)"
  trap 'rm -f "$freeze_sources"' EXIT
  jq -r '.frozen_source_sha256 | keys[]' docs/product/mvp_freeze_manifest.json >"$freeze_sources"
  if grep -Fxf "$freeze_sources" "$changed_files_path" >/dev/null 2>&1; then
    freeze_source_match=true
  fi
fi

if matches '^(docs/product/(mvp_freeze_manifest\.json|mvp_release_and_rollback\.md)|docs/rust-cutover/(tasks|evidence)/MVP-01[23]\.md|scripts/ai/check_mvp_freeze_baseline\.sh|\.github/workflows/rust-cutover-smoke\.yml|scripts/ci/(classify-ci-changes|test-ci-change-classifier)\.sh)$' \
  || [[ "$freeze_source_match" == "true" ]]; then
  mvp_final_acceptance=true
  institution_workbench=true
  strategy_workbench=true
  control_center=true
  mvp_acceptance=true
  mvp_fault_matrix=true
fi

if matches '^(\.github/|\.zizmor\.yml$|scripts/ci/(classify-ci-changes|security-audit-gate|test-ci-change-classifier)\.sh$)'; then
  security_workflow=true
fi

if matches '^(Cargo\.(lock|toml)|crates/(.*/)?Cargo\.toml|deny\.toml$|osv-scanner\.toml$|\.supply-chain/|tools\.toml$|rust-toolchain\.toml$|\.cargo/(config|audit)\.toml$|scripts/(cargo-tool-version|rust-toolchain)\.sh$|\.github/actions/cargo-tool-install/|\.github/workflows/security-audit\.yml$|scripts/ci/(classify-ci-changes|security-audit-gate|test-ci-change-classifier)\.sh$)'; then
  security_dependencies=true
fi

if [[ "${NTPRO_CI_FORCE_FULL_SECURITY:-0}" == "1" ]]; then
  security_workflow=true
  security_dependencies=true
fi

for name in \
  heavy_rust \
  release_verify \
  institution_workbench \
  strategy_workbench \
  control_center \
  mvp_acceptance \
  mvp_fault_matrix \
  mvp_final_acceptance \
  security_workflow \
  security_dependencies \
  frontend_app; do
  value="${!name}"
  printf '%s=%s\n' "$name" "$value" | tee -a "$GITHUB_OUTPUT"
done

if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  {
    echo "## CI change classification"
    echo
    echo "| Class | Run |"
    echo "| --- | --- |"
    for name in \
      heavy_rust \
      release_verify \
      institution_workbench \
      strategy_workbench \
      control_center \
      mvp_acceptance \
      mvp_fault_matrix \
      mvp_final_acceptance \
      security_workflow \
      security_dependencies \
      frontend_app; do
      printf '| `%s` | `%s` |\n' "$name" "${!name}"
    done
  } >>"$GITHUB_STEP_SUMMARY"
fi
