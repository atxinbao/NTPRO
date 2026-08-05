#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

classify() {
  local case_name="$1"
  shift
  local changed="$tmp_dir/${case_name}.files"
  local output="$tmp_dir/${case_name}.output"
  printf '%s\n' "$@" >"$changed"
  GITHUB_OUTPUT="$output" scripts/ci/classify-ci-changes.sh "$changed" >/dev/null
  echo "$output"
}

assert_output() {
  local output="$1"
  local expected="$2"
  if ! grep -Fx "$expected" "$output" >/dev/null; then
    echo "missing classifier output '$expected' in $output" >&2
    cat "$output" >&2
    exit 1
  fi
}

docs_output="$(classify docs-only project.html README.md docs/product/roadmap.md)"
for key in heavy_rust institution_workbench control_center mvp_acceptance mvp_fault_matrix mvp_final_acceptance security_workflow security_dependencies; do
  assert_output "$docs_output" "$key=false"
done

runtime_output="$(classify runtime crates/cli/src/dashboard/server.rs)"
assert_output "$runtime_output" "heavy_rust=true"
assert_output "$runtime_output" "institution_workbench=true"
assert_output "$runtime_output" "control_center=true"
assert_output "$runtime_output" "mvp_acceptance=true"
assert_output "$runtime_output" "mvp_fault_matrix=true"

freeze_output="$(classify freeze docs/product/mvp_freeze_manifest.json)"
for key in institution_workbench control_center mvp_acceptance mvp_fault_matrix mvp_final_acceptance; do
  assert_output "$freeze_output" "$key=true"
done

cargo_output="$(classify cargo Cargo.lock)"
assert_output "$cargo_output" "heavy_rust=true"
assert_output "$cargo_output" "security_dependencies=true"
assert_output "$cargo_output" "security_workflow=false"

workflow_output="$(classify workflow .github/workflows/rust-cutover-smoke.yml)"
assert_output "$workflow_output" "release_verify=true"
assert_output "$workflow_output" "mvp_final_acceptance=true"
assert_output "$workflow_output" "security_workflow=true"

security_output="$(classify security-workflow .github/workflows/security-audit.yml)"
assert_output "$security_output" "security_workflow=true"
assert_output "$security_output" "security_dependencies=true"

forced_changed="$tmp_dir/forced.files"
forced_output="$tmp_dir/forced.output"
printf '%s\n' docs/README.md >"$forced_changed"
NTPRO_CI_FORCE_FULL_SECURITY=1 GITHUB_OUTPUT="$forced_output" \
  scripts/ci/classify-ci-changes.sh "$forced_changed" >/dev/null
assert_output "$forced_output" "security_workflow=true"
assert_output "$forced_output" "security_dependencies=true"

echo "ci_change_classifier_selftest=pass cases=7"
