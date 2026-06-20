#!/usr/bin/env bash
set -euo pipefail

# V120-003: response-shape validation for production read-only artifacts.
# This script is CI-safe and does not open production network.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

cargo test -p nautilus-cli production_account_snapshot_shape --lib
cargo test -p nautilus-cli production_account_snapshot_online_invalid_shape --lib

SNAPSHOT_ROOT="${NTPRO_V12_RESPONSE_SHAPE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v12-response-shape.XXXXXX")}"
NTPRO_V12_ACCOUNT_SNAPSHOT_ROOT="$SNAPSHOT_ROOT/account-snapshot" \
  scripts/ai/verify_v12_authenticated_account_snapshot_online_read.sh

if rg -n \
  'raw_account_response_recorded:\s*true|raw_balances_recorded:\s*true|raw_permissions_recorded:\s*true' \
  crates/cli/src/live.rs docs/rust-cutover/release/v0_12_0_response_shape.md scripts/ai/verify_v12_response_shape.sh
then
  echo "v12 response shape redaction flags must not be true" >&2
  exit 1
fi

PREFLIGHT_JSON="$SNAPSHOT_ROOT/account-snapshot/command-output/manual-preflight-account-snapshot.json"
python3 - "$PREFLIGHT_JSON" <<'PY'
import json
import sys
from pathlib import Path

report = json.loads(Path(sys.argv[1]).read_text())

def require(condition, message):
    if not condition:
        raise SystemExit(message)

summary = report["response_shape_summary"]
require(summary["raw_account_response_recorded"] is False, summary)
require(summary["raw_balances_recorded"] is False, summary)
require(summary["raw_permissions_recorded"] is False, summary)
require(report["production_order_mutation_attempted"] is False, report)
require(report["dashboard_order_controls_enabled"] is False, report)
PY

echo "v12_response_shape status=ok root=$SNAPSHOT_ROOT"
