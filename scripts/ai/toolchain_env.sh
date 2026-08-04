#!/usr/bin/env bash
# 在执行 Cargo 前 source 本文件。rust-toolchain.toml 是唯一版本权威。

NTPRO_TOOLCHAIN_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
NTPRO_CANONICAL_RUST_TOOLCHAIN="$(bash "$NTPRO_TOOLCHAIN_ROOT/scripts/rust-toolchain.sh")"
NTPRO_CARGO_RUST_VERSION="$(
  awk -F'"' '/^[[:space:]]*rust-version[[:space:]]*=/{print $2; exit}' \
    "$NTPRO_TOOLCHAIN_ROOT/Cargo.toml"
)"

if [ -z "$NTPRO_CARGO_RUST_VERSION" ]; then
  echo "Cargo.toml workspace rust-version is missing" >&2
  exit 1
fi

if [ "$NTPRO_CANONICAL_RUST_TOOLCHAIN" != "$NTPRO_CARGO_RUST_VERSION" ]; then
  echo "NTPRO Rust toolchain drift detected" >&2
  echo "rust-toolchain.toml: $NTPRO_CANONICAL_RUST_TOOLCHAIN" >&2
  echo "Cargo.toml rust-version: $NTPRO_CARGO_RUST_VERSION" >&2
  exit 1
fi

if [ -n "${NTPRO_RUST_TOOLCHAIN:-}" ] \
  && [ "$NTPRO_RUST_TOOLCHAIN" != "$NTPRO_CANONICAL_RUST_TOOLCHAIN" ]; then
  echo "NTPRO_RUST_TOOLCHAIN cannot override rust-toolchain.toml" >&2
  echo "requested: $NTPRO_RUST_TOOLCHAIN" >&2
  echo "required: $NTPRO_CANONICAL_RUST_TOOLCHAIN" >&2
  exit 1
fi

NTPRO_RUST_TOOLCHAIN="$NTPRO_CANONICAL_RUST_TOOLCHAIN"
NTPRO_REQUIRED_RUSTC_PREFIX="rustc $NTPRO_RUST_TOOLCHAIN"
NTPRO_REQUIRED_CARGO_PREFIX="cargo $NTPRO_RUST_TOOLCHAIN"

if ! command -v rustup >/dev/null 2>&1; then
  echo "rustup is required for NTPRO verification" >&2
  exit 127
fi

NTPRO_RUSTC="$(rustup which rustc --toolchain "$NTPRO_RUST_TOOLCHAIN")"
NTPRO_CARGO="$(rustup which cargo --toolchain "$NTPRO_RUST_TOOLCHAIN")"
NTPRO_TOOLCHAIN_BIN="$(dirname "$NTPRO_CARGO")"

if [ ! -x "$NTPRO_RUSTC" ] || [ ! -x "$NTPRO_CARGO" ]; then
  echo "NTPRO Rust toolchain is missing: $NTPRO_RUST_TOOLCHAIN" >&2
  echo "Install it with: rustup toolchain install $NTPRO_RUST_TOOLCHAIN" >&2
  exit 1
fi

NTPRO_RUSTC_VERSION="$("$NTPRO_RUSTC" --version)"
case "$NTPRO_RUSTC_VERSION" in
  "$NTPRO_REQUIRED_RUSTC_PREFIX"*) ;;
  *)
    echo "NTPRO verification requires $NTPRO_REQUIRED_RUSTC_PREFIX" >&2
    echo "Resolved rustc: $NTPRO_RUSTC_VERSION at $NTPRO_RUSTC" >&2
    exit 1
    ;;
esac

NTPRO_CARGO_VERSION="$("$NTPRO_CARGO" --version)"
case "$NTPRO_CARGO_VERSION" in
  "$NTPRO_REQUIRED_CARGO_PREFIX"*) ;;
  *)
    echo "NTPRO verification requires $NTPRO_REQUIRED_CARGO_PREFIX" >&2
    echo "Resolved cargo: $NTPRO_CARGO_VERSION at $NTPRO_CARGO" >&2
    exit 1
    ;;
esac

export RUSTC="$NTPRO_RUSTC"
export CARGO="$NTPRO_CARGO"
export RUSTUP_TOOLCHAIN="$NTPRO_RUST_TOOLCHAIN"
export PATH="$NTPRO_TOOLCHAIN_BIN:$PATH"
