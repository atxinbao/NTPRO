#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if [[ -n "${NTPRO_GOVERNANCE_BIN:-}" ]]; then
  exec "$NTPRO_GOVERNANCE_BIN" "$@"
fi

exec cargo run --quiet --locked -p ntpro-governance -- "$@"
