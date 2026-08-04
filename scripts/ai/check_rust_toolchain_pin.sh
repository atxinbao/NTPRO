#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

source scripts/ai/toolchain_env.sh

[[ "$(command -v cargo)" == "$NTPRO_CARGO" ]] || {
  echo "cargo PATH did not resolve to the pinned toolchain" >&2
  exit 1
}
[[ "$(command -v rustc)" == "$NTPRO_RUSTC" ]] || {
  echo "rustc PATH did not resolve to the pinned toolchain" >&2
  exit 1
}
[[ "$(cargo --version)" == "$NTPRO_REQUIRED_CARGO_PREFIX"* ]] || exit 1
[[ "$(rustc --version)" == "$NTPRO_REQUIRED_RUSTC_PREFIX"* ]] || exit 1

expected_direct_cargo_scripts="$(printf '%s\n' \
  scripts/ai/check_product_path_lints.sh \
  scripts/ai/check_rust_only_runtime.sh \
  scripts/ai/ntpro_governance.sh \
  scripts/ai/run_golden_traces.sh \
  scripts/ai/v02_two_node_supervisor_smoke.sh \
  scripts/ai/verify_cli_help.sh \
  scripts/ai/verify_fast.sh \
  scripts/ai/verify_full.sh \
  scripts/ai/verify_release.sh \
  scripts/ci/check-crates-io-trusted-publishing.sh \
  scripts/ci/install-nautilus-cli.sh \
  scripts/ci/publish-cargo-crates.sh \
  scripts/cli/install.sh \
  scripts/clippy-changed.sh \
  scripts/crate-test-features.sh \
  scripts/doc-changed.sh \
  | sort)"
actual_direct_cargo_scripts="$(
  find scripts -type f \( -name '*.sh' -o -name '*.bash' \) -print0 \
    | xargs -0 grep -El '(^|[;&|()[:space:]])cargo([[:space:]]+\+[^[:space:]]+)?[[:space:]]+(audit|bench|binstall|build|check|clippy|deny|doc|fetch|fmt|hack|install|llvm-cov|metadata|miri|nextest|publish|run|search|test|update|upgrade|vet)' \
    | sort
)"

if [ "$actual_direct_cargo_scripts" != "$expected_direct_cargo_scripts" ]; then
  echo "direct Cargo script inventory drift detected" >&2
  echo "expected:" >&2
  printf '%s\n' "$expected_direct_cargo_scripts" >&2
  echo "actual:" >&2
  printf '%s\n' "$actual_direct_cargo_scripts" >&2
  exit 1
fi

direct_cargo_scripts=0
cargo_command_pattern='(^|[;&|()[:space:]])cargo([[:space:]]+\+[^[:space:]]+)?[[:space:]]+(audit|bench|binstall|build|check|clippy|deny|doc|fetch|fmt|hack|install|llvm-cov|metadata|miri|nextest|publish|run|search|test|update|upgrade|vet)'
while IFS= read -r script; do
  direct_cargo_scripts=$((direct_cargo_scripts + 1))
  source_line="$(grep -nE '^[[:space:]]*source .*toolchain_env\.sh' "$script" | head -n 1 | cut -d: -f1)"
  cargo_line="$(awk -v pattern="$cargo_command_pattern" '
    /^[[:space:]]*#/ { next }
    $0 ~ pattern { print NR; exit }
  ' "$script")"
  if [ -z "$source_line" ] || [ -z "$cargo_line" ] || [ "$source_line" -ge "$cargo_line" ]; then
    echo "direct Cargo script must load toolchain_env.sh before its first Cargo command: $script" >&2
    exit 1
  fi
done <<EOF
$actual_direct_cargo_scripts
EOF

if grep -ERq --include='*.yml' "toolchain:[[:space:]]*['\"]?[0-9]+\.[0-9]+\.[0-9]+" .github/workflows; then
  echo "workflow contains a duplicated literal Rust toolchain version" >&2
  exit 1
fi

workflow_toolchain_bindings=0
expected_workflow_block="$(printf '%s\n' \
  '      - name: Resolve pinned Rust toolchain' \
  '        id: rust' \
  '        run: |' \
  '          set -euo pipefail' \
  '          toolchain="$(bash scripts/rust-toolchain.sh)"' \
  '          [[ -n "$toolchain" ]]' \
  "          printf 'toolchain=%s\\n' \"\$toolchain\" >>\"\$GITHUB_OUTPUT\"")"
expected_setup_consumer="$(printf '%s\n' \
  '      - uses: actions-rust-lang/setup-rust-toolchain@2b1f5e9b395427c92ee4e3331786ca3c37afe2d7 # v1.16.0' \
  '        with:' \
  '          toolchain: ${{ steps.rust.outputs.toolchain }}')"
for workflow in \
  .github/workflows/rust-cutover-smoke.yml \
  .github/workflows/backend-performance.yml \
  .github/workflows/release-tag.yml \
  .github/workflows/release-publish.yml; do
  workflow_toolchain_bindings=$((workflow_toolchain_bindings + 1))
  actual_workflow_block="$(awk '
    /^      - name: Resolve pinned Rust toolchain$/ { capture = 1 }
    capture { print }
    capture && /printf '\''toolchain=%s\\n'\'' "\$toolchain" >>"\$GITHUB_OUTPUT"/ { exit }
  ' "$workflow")"
  if [ "$actual_workflow_block" != "$expected_workflow_block" ]; then
    echo "workflow toolchain resolution is not one ordered fail-closed block: $workflow" >&2
    exit 1
  fi
  actual_setup_consumer="$(awk '
    /^      - uses: actions-rust-lang\/setup-rust-toolchain@/ { capture = 1 }
    capture { print }
    capture && /^          toolchain:/ { exit }
  ' "$workflow")"
  if [ "$actual_setup_consumer" != "$expected_setup_consumer" ]; then
    echo "workflow setup action does not consume the canonical toolchain output: $workflow" >&2
    exit 1
  fi
done

if ! grep -Eq '^override CARGO := \$\(NTPRO_RUSTUP_BIN\) run \$\(NTPRO_RUST_TOOLCHAIN\) cargo$' Makefile \
  || ! grep -Eq '^override CARGO_NIGHTLY := \$\(NTPRO_RUSTUP_BIN\) run nightly cargo$' Makefile; then
  echo "Makefile does not preserve canonical and nightly toolchain bindings" >&2
  exit 1
fi

tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/ntpro-toolchain-pin.XXXXXX")"
trap 'rm -rf "$tmpdir"' EXIT

cp scripts/cli/install.sh "$tmpdir/install.sh"
set +e
standalone_help_output="$(cd "$tmpdir" && bash install.sh -h 2>&1)"
standalone_help_status=$?
set -e
if [ "$standalone_help_status" -ne 1 ] \
  || [ "$standalone_help_output" != 'Usage: install.sh [-b /install/dir]' ]; then
  echo "standalone CLI installer depends on repository-only files before source fallback" >&2
  printf '%s\n' "$standalone_help_output" >&2
  exit 1
fi

if NTPRO_RUST_TOOLCHAIN=1.87.0 \
  bash -c 'source scripts/ai/toolchain_env.sh' >/dev/null 2>&1; then
  echo "toolchain negative selftest accepted environment override" >&2
  exit 1
fi

mkdir -p "$tmpdir/stable/scripts"
cp scripts/rust-toolchain.sh "$tmpdir/stable/scripts/rust-toolchain.sh"
printf '%s\n' '[toolchain]' 'channel = "stable"' >"$tmpdir/stable/rust-toolchain.toml"
if bash "$tmpdir/stable/scripts/rust-toolchain.sh" >/dev/null 2>&1; then
  echo "toolchain negative selftest accepted floating stable" >&2
  exit 1
fi

mkdir -p "$tmpdir/mismatch/scripts/ai"
cp scripts/rust-toolchain.sh "$tmpdir/mismatch/scripts/rust-toolchain.sh"
cp scripts/ai/toolchain_env.sh "$tmpdir/mismatch/scripts/ai/toolchain_env.sh"
printf '%s\n' '[toolchain]' 'channel = "1.87.0"' >"$tmpdir/mismatch/rust-toolchain.toml"
printf '%s\n' '[workspace]' 'members = []' '' '[workspace.package]' 'rust-version = "1.95.0"' \
  >"$tmpdir/mismatch/Cargo.toml"
if bash -c 'source "$1"' _ "$tmpdir/mismatch/scripts/ai/toolchain_env.sh" >/dev/null 2>&1; then
  echo "toolchain negative selftest accepted Cargo.toml mismatch" >&2
  exit 1
fi

echo "rust_toolchain_negative_selftest=pass cases=3"
echo "rust_toolchain_standalone_installer_selftest=pass cases=1"

echo "rust_toolchain_pin=pass toolchain=$NTPRO_RUST_TOOLCHAIN cargo=$NTPRO_CARGO rustc=$NTPRO_RUSTC direct_cargo_scripts=$direct_cargo_scripts workflow_bindings=$workflow_toolchain_bindings workflow_literals=0"
