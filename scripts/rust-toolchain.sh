#!/bin/bash
set -euo pipefail

# Resolve rust-toolchain.toml relative to this script's location
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLCHAIN_FILE="${NTPRO_RUST_TOOLCHAIN_FILE:-${SCRIPT_DIR}/../rust-toolchain.toml}"

# Check that rust-toolchain.toml exists
if [[ ! -f "$TOOLCHAIN_FILE" ]]; then
  echo "Error: rust-toolchain.toml not found at $TOOLCHAIN_FILE" >&2
  exit 1
fi

# Extract the pinned channel from rustup's standard toolchain file.
VERSION=$(awk -F'"' '/^[[:space:]]*channel[[:space:]]*=/{print $2; exit}' "$TOOLCHAIN_FILE")

# Require an exact semantic version so "stable" cannot silently drift.
if [[ -z "$VERSION" ]]; then
  echo "Error: Could not extract toolchain channel from $TOOLCHAIN_FILE" >&2
  exit 1
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: Rust toolchain channel must be an exact version, found '$VERSION'" >&2
  exit 1
fi

# Output version (without trailing newline for consistency)
echo -n "$VERSION"
