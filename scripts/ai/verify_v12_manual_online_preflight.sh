#!/usr/bin/env bash
set -euo pipefail

# V120-008: v0.12 owner-gated production online read-only preflight.
# This default gate is intentionally offline. It proves that manual-online
# requests still fail closed unless the final owner-controlled online env is set.
# It does not open production network connections and does not read real
# credentials.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V12_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V12_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V12_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

PREFLIGHT_ROOT="${NTPRO_V12_MANUAL_ONLINE_PREFLIGHT_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v12-manual-online-preflight.XXXXXX")}"
PUBLIC_ROOT="$PREFLIGHT_ROOT/public"
ACCOUNT_ROOT="$PREFLIGHT_ROOT/account"

export NTPRO_V12_SKIP_BUILD=1
export NTPRO_V12_NAUTILUS_BIN="$NAUTILUS_BIN"

echo "== v12 manual online preflight: public production GET remains blocked without final online env =="
env -u NTPRO_V12_MANUAL_ONLINE \
  NTPRO_V12_PUBLIC_READ_ROOT="$PUBLIC_ROOT" \
  scripts/ai/verify_v12_public_online_read_probe.sh

echo "== v12 manual online preflight: authenticated account GET remains blocked without final online env =="
env -u NTPRO_V12_MANUAL_ONLINE \
  NTPRO_V12_ACCOUNT_SNAPSHOT_ROOT="$ACCOUNT_ROOT" \
  scripts/ai/verify_v12_authenticated_account_snapshot_online_read.sh

PUBLIC_PREFLIGHT_JSON="$PUBLIC_ROOT/command-output/manual-preflight-public-read-probe.json"
ACCOUNT_PREFLIGHT_JSON="$ACCOUNT_ROOT/command-output/manual-preflight-account-snapshot.json"

python3 - "$PUBLIC_PREFLIGHT_JSON" "$ACCOUNT_PREFLIGHT_JSON" <<'PY'
import json
import sys
from pathlib import Path

public_report = json.loads(Path(sys.argv[1]).read_text())
account_report = json.loads(Path(sys.argv[2]).read_text())

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(public_report["schema_version"] == "ntpro.v120_production_public_online_read_probe.v1", public_report)
require(public_report["status"] == "blocked_missing_manual_online_gate", public_report)
require(public_report["missing_env_vars"] == ["NTPRO_V12_MANUAL_ONLINE"], public_report)
require(public_report["network_attempted"] is False, public_report)
require(public_report["production_public_online_read_attempted"] is False, public_report)
require(public_report["production_order_submission_attempted"] is False, public_report)
require(public_report["production_order_mutation_attempted"] is False, public_report)
require(public_report["dashboard_order_controls_enabled"] is False, public_report)

require(account_report["schema_version"] == "ntpro.v120_authenticated_account_snapshot_online_read.v1", account_report)
require(account_report["status"] == "blocked_missing_manual_online_gate", account_report)
require(account_report["missing_env_vars"] == ["NTPRO_V12_MANUAL_ONLINE"], account_report)
require(account_report["network_attempted"] is False, account_report)
require(account_report["account_read_attempted"] is False, account_report)
require(account_report["signature_recorded"] is False, account_report)
require(account_report["signed_query_recorded"] is False, account_report)
require(account_report["signed_url_recorded"] is False, account_report)
require(account_report["production_order_submission_attempted"] is False, account_report)
require(account_report["production_order_mutation_attempted"] is False, account_report)
require(account_report["dashboard_order_controls_enabled"] is False, account_report)
PY

echo "v12_manual_online_preflight status=ok root=$PREFLIGHT_ROOT network_attempted=false owner_gated_online_not_required_for_ci=true"
