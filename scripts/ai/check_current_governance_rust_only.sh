#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

authority_files=(
  .github/workflows/rust-cutover-smoke.yml
  .github/workflows/release-publish.yml
  scripts/ai/toolchain_env.sh
  scripts/ai/ntpro_governance.sh
  scripts/ai/verify_fast.sh
  scripts/ai/check_release_surface_current.sh
  scripts/ai/check_docs_examples_governance.sh
  scripts/ai/check_rust_examples.sh
  scripts/ai/check_backend_freeze_baseline.sh
  scripts/ai/check_rust_only_runtime.sh
  scripts/ai/check_cython_removed.sh
  scripts/ai/check_github_release_published.sh
  scripts/ai/publish_ntpro_release_after_gate.sh
  scripts/ai/verify_release_publish_after_gate.sh
  scripts/ai/check_current_governance_rust_only.sh
)

for path in "${authority_files[@]}"; do
  [[ -f "$path" ]] || {
    echo "current governance Rust-only guard: missing authority file: $path" >&2
    exit 1
  }
done

scan_python_execution() {
  local heredoc_pattern='python3?([^[:alnum:]_]|$).*<<' # current-governance-pattern-definition
  local command_pattern='(^|[;&|])[[:space:]]*(env[[:space:]]+)?([A-Za-z_][A-Za-z0-9_]*=[^[:space:]]+[[:space:]]+)*([^[:space:]]*/)?(python3?|uv|pytest|ruff|pip-audit)([[:space:]]|$)' # current-governance-pattern-definition
  local tooling_pattern='(^|[[:space:]])(uv[[:space:]]+run|pytest|ruff|pip-audit)([[:space:]]|$)' # current-governance-pattern-definition
  grep -En -e "$heredoc_pattern" -e "$command_pattern" -e "$tooling_pattern" -- "$@" \
    | grep -Ev 'current-governance-pattern-definition'
}

if matches="$(scan_python_execution "${authority_files[@]}" || true)" && [[ -n "$matches" ]]; then
  printf '%s\n' "$matches" >&2
  echo "current governance Rust-only guard: Python tooling execution remains" >&2
  exit 1
fi

negative_cases=0
if [[ "${NTPRO_CURRENT_GOVERNANCE_NEGATIVE_SELFTEST:-1}" == "1" ]]; then
  fixture_direct="$(mktemp "${TMPDIR:-/tmp}/ntpro-current-governance-direct.XXXXXX")"
  fixture_prefixed="$(mktemp "${TMPDIR:-/tmp}/ntpro-current-governance-prefixed.XXXXXX")"
  trap 'rm -f "$fixture_direct" "$fixture_prefixed"' EXIT
  printf '%s\n' "python3 <<'PY'" "print('forbidden')" "PY" >"$fixture_direct" # current-governance-pattern-definition
  printf '%s\n' "env MODE=test .venv/bin/python script.py" >"$fixture_prefixed" # current-governance-pattern-definition
  for fixture in "$fixture_direct" "$fixture_prefixed"; do
    if ! scan_python_execution "$fixture" >/dev/null; then
      echo "current governance Rust-only guard: negative selftest did not detect Python: $fixture" >&2
      exit 1
    fi
    negative_cases=$((negative_cases + 1))
  done
fi

echo "current_governance_rust_only=pass authority_files=${#authority_files[@]} negative_cases=$negative_cases"
