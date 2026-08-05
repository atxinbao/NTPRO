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
for key in heavy_rust institution_workbench strategy_workbench control_center mvp_acceptance mvp_fault_matrix mvp_final_acceptance security_workflow security_dependencies frontend_app; do
  assert_output "$docs_output" "$key=false"
done

runtime_output="$(classify runtime crates/cli/src/dashboard/server.rs)"
assert_output "$runtime_output" "heavy_rust=true"
assert_output "$runtime_output" "institution_workbench=true"
assert_output "$runtime_output" "strategy_workbench=true"
assert_output "$runtime_output" "control_center=true"
assert_output "$runtime_output" "mvp_acceptance=true"
assert_output "$runtime_output" "mvp_fault_matrix=true"

freeze_output="$(classify freeze docs/product/mvp_freeze_manifest.json)"
for key in institution_workbench strategy_workbench control_center mvp_acceptance mvp_fault_matrix mvp_final_acceptance; do
  assert_output "$freeze_output" "$key=true"
done

strategy_output="$(classify strategy-workbench crates/cli/src/dashboard/strategy_workbench.rs)"
assert_output "$strategy_output" "heavy_rust=true"
assert_output "$strategy_output" "strategy_workbench=true"
assert_output "$strategy_output" "institution_workbench=false"
assert_output "$strategy_output" "control_center=false"

frontend_output="$(classify frontend-app apps/strategy-workbench/src/pages/OverviewPage.tsx apps/strategy-workbench/package-lock.json)"
assert_output "$frontend_output" "frontend_app=true"
assert_output "$frontend_output" "heavy_rust=false"
assert_output "$frontend_output" "strategy_workbench=false"
assert_output "$frontend_output" "mvp_acceptance=false"

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

fixture="$tmp_dir/security-fixture"
mkdir -p "$fixture/scripts/ci" "$fixture/docs/product" "$fixture/docs" "$fixture/crates"
cp scripts/ci/classify-ci-changes.sh scripts/ci/security-audit-gate.sh "$fixture/scripts/ci/"
printf '%s\n' '{"frozen_source_sha256":{"crates/frozen.rs":"fixture"}}' \
  >"$fixture/docs/product/mvp_freeze_manifest.json"
printf '%s\n' 'rules: {}' >"$fixture/.zizmor.yml"
printf '%s\n' '# lock' >"$fixture/Cargo.lock"
printf '%s\n' '// frozen source' >"$fixture/crates/frozen.rs"
git -C "$fixture" init -q -b main
git -C "$fixture" config user.name "CI classifier test"
git -C "$fixture" config user.email "ci-classifier@example.invalid"
git -C "$fixture" add .
git -C "$fixture" commit -q -m base
base_sha="$(git -C "$fixture" rev-parse HEAD)"
git -C "$fixture" update-ref refs/remotes/origin/main "$base_sha"
git -C "$fixture" switch -q -c feature

run_security_gate() {
  local case_name="$1"
  local event_name="$2"
  local pr_head_sha="$3"
  local push_before_sha="$4"
  local push_after_sha="$5"
  local pr_merge_sha="${6:-}"
  local output="$tmp_dir/security-${case_name}.output"
  : >"$output"
  (
    cd "$fixture"
    GITHUB_OUTPUT="$output" \
      EVENT_NAME="$event_name" \
      PR_BASE_REF=main \
      PR_HEAD_SHA="$pr_head_sha" \
      PR_MERGE_SHA="$pr_merge_sha" \
      PUSH_BEFORE_SHA="$push_before_sha" \
      PUSH_AFTER_SHA="$push_after_sha" \
      scripts/ci/security-audit-gate.sh >/dev/null
  )
  echo "$output"
}

printf '%s\n' 'docs only' >"$fixture/docs/guide.md"
git -C "$fixture" add docs/guide.md
git -C "$fixture" commit -q -m docs
docs_head="$(git -C "$fixture" rev-parse HEAD)"
event_output="$(run_security_gate pr-docs pull_request "$docs_head" '' '')"
assert_output "$event_output" "security_workflow=false"
assert_output "$event_output" "security_dependencies=false"

git -C "$fixture" mv .zizmor.yml docs/zizmor.txt
git -C "$fixture" mv crates/frozen.rs docs/frozen.rs
git -C "$fixture" commit -q -m rename
rename_head="$(git -C "$fixture" rev-parse HEAD)"
event_output="$(run_security_gate pr-rename pull_request "$rename_head" '' '')"
assert_output "$event_output" "security_workflow=true"
assert_output "$event_output" "mvp_final_acceptance=true"

printf '%s\n' '# dependency change' >>"$fixture/Cargo.lock"
git -C "$fixture" add Cargo.lock
git -C "$fixture" commit -q -m dependency
dependency_head="$(git -C "$fixture" rev-parse HEAD)"
event_output="$(run_security_gate push-dependency push '' "$rename_head" "$dependency_head")"
assert_output "$event_output" "security_workflow=false"
assert_output "$event_output" "security_dependencies=true"

merge_tree="$(git -C "$fixture" rev-parse "${dependency_head}^{tree}")"
merge_sha="$(printf '%s\n' 'synthetic pull request merge' | git -C "$fixture" commit-tree \
  "$merge_tree" -p "$base_sha" -p "$dependency_head")"
event_output="$(run_security_gate pr-merge-depth-two pull_request "$dependency_head" '' '' "$merge_sha")"
assert_output "$event_output" "security_workflow=true"
assert_output "$event_output" "security_dependencies=true"

invalid_merge_sha="$(printf '%s\n' 'invalid synthetic merge' | git -C "$fixture" commit-tree \
  "$merge_tree" -p "$dependency_head" -p "$base_sha")"
event_output="$(run_security_gate pr-invalid-merge pull_request "$dependency_head" '' '' "$invalid_merge_sha")"
assert_output "$event_output" "security_workflow=true"
assert_output "$event_output" "security_dependencies=true"

for event_name in schedule workflow_dispatch; do
  event_output="$(run_security_gate "$event_name" "$event_name" '' '' '')"
  assert_output "$event_output" "security_workflow=true"
  assert_output "$event_output" "security_dependencies=true"
done

zero_sha="0000000000000000000000000000000000000000"
event_output="$(run_security_gate zero-base push '' "$zero_sha" "$dependency_head")"
assert_output "$event_output" "security_workflow=true"
assert_output "$event_output" "security_dependencies=true"

missing_sha="1111111111111111111111111111111111111111"
event_output="$(run_security_gate missing-head push '' "$dependency_head" "$missing_sha")"
assert_output "$event_output" "security_workflow=true"
assert_output "$event_output" "security_dependencies=true"

if ! grep -F "group: \${{ github.workflow }}-\${{ github.event_name }}-\${{ github.event_name == 'pull_request' && github.event.pull_request.number || github.run_id }}" \
  .github/workflows/security-audit.yml >/dev/null; then
  echo "security audit concurrency does not isolate event types" >&2
  exit 1
fi
if ! grep -F "cancel-in-progress: \${{ github.event_name == 'pull_request' }}" \
  .github/workflows/security-audit.yml >/dev/null; then
  echo "security audit cancellation must be limited to pull requests" >&2
  exit 1
fi

echo "ci_change_classifier_selftest=pass cases=19"
