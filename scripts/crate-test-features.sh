#!/usr/bin/env bash
# Returns a comma-separated list of all non-default features for a crate.
#
# Usage: scripts/crate-test-features.sh <crate-name>
# Example: scripts/crate-test-features.sh nautilus-live
#   -> ffi,streaming,defi

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/ai/toolchain_env.sh"

if [ $# -ne 1 ]; then
  echo "Usage: $0 <crate-name>" >&2
  exit 1
fi

cargo metadata --no-deps --format-version 1 |
  jq -r --arg p "$1" '
        [.packages[]
         | select(.name == $p)
         | .features
         | keys[]
         | select(. != "default" and . != "python")
        ] | join(",")'
