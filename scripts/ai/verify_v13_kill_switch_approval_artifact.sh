#!/usr/bin/env bash
set -euo pipefail

# V130-004: guarded-live-alpha kill-switch dry-run/manual-approval artifact.
# This script is CI-safe. It writes local artifacts only, opens no production
# network, and never submits, cancels, replaces, amends, retries, or corrects
# production orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

cargo test -p nautilus-cli production_kill_switch_approval_artifact --lib

if [[ "${NTPRO_V13_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V13_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V13_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

ARTIFACT_ROOT="${NTPRO_V13_KILL_SWITCH_APPROVAL_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v13-kill-switch-approval.XXXXXX")}"
OUTPUT_DIR="$ARTIFACT_ROOT/output"
mkdir -p "$OUTPUT_DIR"

ARTIFACT_JSON="$OUTPUT_DIR/kill_switch_approval_artifact.json"
STDOUT_LOG="$OUTPUT_DIR/kill_switch_approval.stdout.log"
STDERR_LOG="$OUTPUT_DIR/kill_switch_approval.stderr.log"

"$NAUTILUS_BIN" live production-kill-switch-approval-artifact \
  --run-id v130-live-alpha-preflight \
  --session-id v130-live-alpha-session \
  --strategy-id ema_cross_btcusdt_v1 \
  --output "$ARTIFACT_JSON" \
  --kill-switch-active true \
  --approval-state approved \
  --manual-approval-id owner-approval-v130-004 \
  --approved-by owner \
  --confirm-dry-run-only \
  --confirm-no-production-mutation \
  --confirm-dashboard-order-controls-disabled \
  >"$STDOUT_LOG" \
  2>"$STDERR_LOG"

if [[ -s "$STDERR_LOG" ]]; then
  echo "v13 kill-switch approval artifact wrote stderr" >&2
  cat "$STDERR_LOG" >&2
  exit 1
fi

python3 - "$ARTIFACT_JSON" "$STDOUT_LOG" <<'PY'
import json
import sys
from pathlib import Path

artifact = json.loads(Path(sys.argv[1]).read_text())
stdout = Path(sys.argv[2]).read_text()

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require("kill_switch_dry_run=true" in stdout, stdout)
require("manual_approval_recorded=true" in stdout, stdout)
require("production_order_mutations_attempted=0" in stdout, stdout)
require("network_attempted=false" in stdout, stdout)
require(artifact["schema_version"] == "ntpro.v130_kill_switch_approval_artifact.v1", artifact)
require(artifact["artifact_type"] == "kill_switch_dry_run_manual_approval", artifact)
require(artifact["status"] == "manual_approval_recorded", artifact)
require(artifact["kill_switch_enabled"] is True, artifact)
require(artifact["kill_switch_active"] is True, artifact)
require(artifact["kill_switch_dry_run"] is True, artifact)
require(artifact["manual_approval_required"] is True, artifact)
require(artifact["manual_approval_recorded"] is True, artifact)
require(artifact["manual_approval_id"] == "owner-approval-v130-004", artifact)
require(artifact["approved_by"] == "owner", artifact)
require(artifact["approval_state"] == "approved", artifact)
require(artifact["approval_artifact_only"] is True, artifact)
require(artifact["owner_approval_required_before_any_mutation"] is True, artifact)
require(artifact["production_order_submission_allowed"] is False, artifact)
require(artifact["production_order_mutation_allowed"] is False, artifact)
require(artifact["production_order_state_reads_allowed"] is False, artifact)
require(artifact["listen_key_lifecycle_allowed"] is False, artifact)
require(artifact["production_order_submissions_attempted"] == 0, artifact)
require(artifact["production_orders_submitted"] == 0, artifact)
require(artifact["production_order_mutations_attempted"] == 0, artifact)
require(artifact["production_order_state_reads_attempted"] == 0, artifact)
require(artifact["listen_key_lifecycle_attempted"] == 0, artifact)
require(artifact["cancel_replace_amend_attempted"] is False, artifact)
require(artifact["actual_submission_count"] == 0, artifact)
require(artifact["automatic_correction_orders_submitted"] == 0, artifact)
require(artifact["dashboard_order_controls_enabled"] is False, artifact)
require(artifact["real_orders_submitted"] is False, artifact)
require(artifact["production_trading_enabled"] is False, artifact)
require(artifact["network_attempted"] is False, artifact)
require(artifact["values_are_exchange_truth"] is False, artifact)
require(artifact["dry_run_confirmed"] is True, artifact)
require(artifact["no_production_mutation_confirmed"] is True, artifact)
require(artifact["dashboard_controls_disabled_confirmed"] is True, artifact)
PY

"$NAUTILUS_BIN" live production-kill-switch-approval-artifact --help >/dev/null

echo "v13_kill_switch_approval_artifact status=ok root=$ARTIFACT_ROOT network_attempted=false production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false"
