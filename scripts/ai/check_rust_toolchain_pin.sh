#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

RUN_NEGATIVE_SELFTEST="${NTPRO_TOOLCHAIN_NEGATIVE_SELFTEST:-1}"
case "$RUN_NEGATIVE_SELFTEST" in
  0) negative_selftest=false ;;
  1) negative_selftest=true ;;
  *)
    echo "NTPRO_TOOLCHAIN_NEGATIVE_SELFTEST must be 0 or 1" >&2
    exit 1
    ;;
esac

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
  if ! rg -q '^source (scripts/ai/toolchain_env\.sh|"\$SCRIPT_ROOT/scripts/ai/toolchain_env\.sh")' "$script"; then
    echo "direct Cargo script does not load toolchain_env.sh: $script" >&2
    exit 1
  fi
done < <(
  rg -l '(^|[[:space:]])cargo[[:space:]]+(bench|build|check|clippy|fmt|metadata|run|test)' \
    scripts/ai --glob '*.sh'
)

if rg -q 'toolchain:[[:space:]]+[0-9]+\.[0-9]+\.[0-9]+' .github/workflows; then
  echo "workflow contains a duplicated literal Rust toolchain version" >&2
  exit 1
fi

if "$negative_selftest"; then
  tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/ntpro-toolchain-pin.XXXXXX")"
  trap 'rm -rf "$tmpdir"' EXIT

  if NTPRO_RUST_TOOLCHAIN=1.87.0 \
    bash -c 'source scripts/ai/toolchain_env.sh' >/dev/null 2>&1; then
    echo "toolchain negative selftest accepted environment override" >&2
    exit 1
  fi

  printf '%s\n' '[toolchain]' 'channel = "stable"' >"$tmpdir/stable.toml"
  if NTPRO_RUST_TOOLCHAIN_FILE="$tmpdir/stable.toml" \
    bash scripts/rust-toolchain.sh >/dev/null 2>&1; then
    echo "toolchain negative selftest accepted floating stable" >&2
    exit 1
  fi

  printf '%s\n' '[toolchain]' 'channel = "1.87.0"' >"$tmpdir/old.toml"
  if NTPRO_RUST_TOOLCHAIN_FILE="$tmpdir/old.toml" \
    bash -c 'source scripts/ai/toolchain_env.sh' >/dev/null 2>&1; then
    echo "toolchain negative selftest accepted Cargo.toml mismatch" >&2
    exit 1
  fi

  echo "rust_toolchain_negative_selftest=pass cases=3"
fi

echo "rust_toolchain_pin=pass toolchain=$NTPRO_RUST_TOOLCHAIN cargo=$NTPRO_CARGO rustc=$NTPRO_RUSTC direct_cargo_scripts=$direct_cargo_scripts workflow_literals=0"
