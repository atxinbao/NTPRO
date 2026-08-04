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

direct_cargo_scripts=0
while IFS= read -r script; do
  direct_cargo_scripts=$((direct_cargo_scripts + 1))
  if ! rg -q 'toolchain_env\.sh' "$script"; then
    echo "direct Cargo script does not load toolchain_env.sh: $script" >&2
    exit 1
  fi
done < <(
  rg -l '(^|[;&|()[:space:]])cargo([[:space:]]+\+[^[:space:]]+)?[[:space:]]+(audit|bench|binstall|build|check|clippy|deny|doc|fetch|fmt|hack|install|llvm-cov|metadata|miri|nextest|publish|run|search|test|update|upgrade|vet)' \
    scripts --glob '*.sh' --glob '*.bash'
)

if rg -q 'toolchain:[[:space:]]+[0-9]+\.[0-9]+\.[0-9]+' .github/workflows; then
  echo "workflow contains a duplicated literal Rust toolchain version" >&2
  exit 1
fi

workflow_toolchain_bindings=0
for workflow in \
  .github/workflows/rust-cutover-smoke.yml \
  .github/workflows/backend-performance.yml \
  .github/workflows/release-tag.yml \
  .github/workflows/release-publish.yml; do
  workflow_toolchain_bindings=$((workflow_toolchain_bindings + 1))
  for required_line in \
    'set -euo pipefail' \
    'toolchain="$(bash scripts/rust-toolchain.sh)"' \
    '[[ -n "$toolchain" ]]' \
    "printf 'toolchain=%s\\n' \"\$toolchain\" >>\"\$GITHUB_OUTPUT\""; do
    if ! rg -Fq "$required_line" "$workflow"; then
      echo "workflow toolchain resolution is not fail closed: $workflow" >&2
      exit 1
    fi
  done
done

if ! rg -q '^override CARGO := \$\(NTPRO_RUSTUP_BIN\) run \$\(NTPRO_RUST_TOOLCHAIN\) cargo$' Makefile \
  || ! rg -q '^override CARGO_NIGHTLY := \$\(NTPRO_RUSTUP_BIN\) run nightly cargo$' Makefile; then
  echo "Makefile does not preserve canonical and nightly toolchain bindings" >&2
  exit 1
fi

tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/ntpro-toolchain-pin.XXXXXX")"
trap 'rm -rf "$tmpdir"' EXIT

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

echo "rust_toolchain_pin=pass toolchain=$NTPRO_RUST_TOOLCHAIN cargo=$NTPRO_CARGO rustc=$NTPRO_RUSTC direct_cargo_scripts=$direct_cargo_scripts workflow_bindings=$workflow_toolchain_bindings workflow_literals=0"
