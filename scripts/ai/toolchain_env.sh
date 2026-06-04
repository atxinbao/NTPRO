#!/usr/bin/env bash
# Source this file from verification scripts before running cargo.

NTPRO_RUST_TOOLCHAIN="${NTPRO_RUST_TOOLCHAIN:-1.95.0}"
NTPRO_REQUIRED_RUSTC_PREFIX="${NTPRO_REQUIRED_RUSTC_PREFIX:-rustc 1.95.0}"

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

export RUSTC="$NTPRO_RUSTC"
export PATH="$NTPRO_TOOLCHAIN_BIN:$PATH"
