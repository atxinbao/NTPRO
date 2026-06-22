#!/usr/bin/env bash
set -euo pipefail

# V150-005: one-time short-lived manual approval lifecycle for production
# live-alpha request preview only. Safe for local development and CI. It never
# opens network connections, calls execution adapters, or submits production
# orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V15_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V15_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V15_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

APPROVAL_ROOT="${NTPRO_V15_APPROVAL_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v15-approval.XXXXXX")}"
OUTPUT_DIR="$APPROVAL_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

ORDER_GATE="$OUTPUT_DIR/live-alpha-order-gate.json"
SYNTHETIC_API_KEY="ntpro_v150005_synthetic_api_key_value"
SYNTHETIC_API_SECRET="ntpro_v150005_synthetic_api_secret_value"

"$NAUTILUS_BIN" live production-live-alpha-dry-run-order-gate \
  --run-id v150-approval-lifecycle \
  --session-id session-v150 \
  --strategy-id ema_cross_btcusdt_v1 \
  --symbol BTCUSDT \
  --side BUY \
  --order-type LIMIT \
  --quantity 0.001 \
  --notional 10.00 \
  --output "$ORDER_GATE" \
  --allow-production-live-alpha-dry-run \
  --confirm-owner-approved-dry-run \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-no-execution-adapter-call \
  --confirm-no-listen-key-lifecycle \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-real-funds >/dev/null

write_approval() {
  local name="$1"
  local state="$2"
  local run_id="$3"
  local symbol="$4"
  local notional="$5"
  local now_ms="$6"
  local expires_ms="$7"
  local output="$OUTPUT_DIR/manual-approval-$name.json"
  local cmd=(
    "$NAUTILUS_BIN" live production-live-alpha-manual-approval-lifecycle
    --run-id "$run_id"
    --strategy-id ema_cross_btcusdt_v1
    --symbol "$symbol"
    --notional "$notional"
    --approval-state "$state"
    --now-unix-ms "$now_ms"
    --expires-at-unix-ms "$expires_ms"
    --output "$output"
    --confirm-dry-run-request-preview-only
    --confirm-one-time-approval
    --confirm-no-production-mutation
    --confirm-dashboard-order-controls-disabled
  )
  if [[ "$state" != "pending" ]]; then
    cmd+=(--manual-approval-id "owner-approval-v150-005-$name" --approved-by owner)
  fi
  "${cmd[@]}" >/dev/null
  printf '%s\n' "$output"
}

run_request_preview() {
  local name="$1"
  local approval="$2"
  local output="$OUTPUT_DIR/request-preview-$name.json"
  NTPRO_V150005_API_KEY="$SYNTHETIC_API_KEY" \
  NTPRO_V150005_API_SECRET="$SYNTHETIC_API_SECRET" \
    "$NAUTILUS_BIN" live production-live-alpha-order-request-preview \
      --run-id v150-request-preview \
      --order-gate "$ORDER_GATE" \
      --manual-approval-lifecycle "$approval" \
      --endpoint-path /api/v3/order \
      --price 10000.00 \
      --time-in-force GTC \
      --timestamp-ms 1718400000000 \
      --recv-window-ms 5000 \
      --api-key-env NTPRO_V150005_API_KEY \
      --api-secret-env NTPRO_V150005_API_SECRET \
      --output "$output" \
      --allow-production-live-alpha-request-preview \
      --confirm-owner-approved-request-preview \
      --confirm-memory-only-signature \
      --confirm-no-production-order-submission \
      --confirm-no-production-order-mutation \
      --confirm-no-execution-adapter-call \
      --confirm-no-network \
      --confirm-no-listen-key-lifecycle \
      --confirm-dashboard-order-controls-disabled \
      --confirm-no-real-funds >/dev/null
  printf '%s\n' "$output"
}

PENDING_APPROVAL="$(write_approval pending pending v150-request-preview BTCUSDT 10.00 1718400000000 1718400060000)"
EXPIRED_APPROVAL="$(write_approval expired expired v150-request-preview BTCUSDT 10.00 1718400070000 1718400060000)"
REVOKED_APPROVAL="$(write_approval revoked revoked v150-request-preview BTCUSDT 10.00 1718400000000 1718400060000)"
USED_APPROVAL="$(write_approval used used v150-request-preview BTCUSDT 10.00 1718400000000 1718400060000)"
RUN_MISMATCH_APPROVAL="$(write_approval run-mismatch approved wrong-run-id BTCUSDT 10.00 1718400000000 1718400060000)"
SYMBOL_MISMATCH_APPROVAL="$(write_approval symbol-mismatch approved v150-request-preview ETHUSDT 10.00 1718400000000 1718400060000)"
NOTIONAL_MISMATCH_APPROVAL="$(write_approval notional-mismatch approved v150-request-preview BTCUSDT 11.00 1718400000000 1718400060000)"
VALID_APPROVAL="$(write_approval valid approved v150-request-preview BTCUSDT 10.00 1718400000000 1718400060000)"

PENDING_PREVIEW="$(run_request_preview pending "$PENDING_APPROVAL")"
EXPIRED_PREVIEW="$(run_request_preview expired "$EXPIRED_APPROVAL")"
REVOKED_PREVIEW="$(run_request_preview revoked "$REVOKED_APPROVAL")"
USED_PREVIEW="$(run_request_preview used "$USED_APPROVAL")"
RUN_MISMATCH_PREVIEW="$(run_request_preview run-mismatch "$RUN_MISMATCH_APPROVAL")"
SYMBOL_MISMATCH_PREVIEW="$(run_request_preview symbol-mismatch "$SYMBOL_MISMATCH_APPROVAL")"
NOTIONAL_MISMATCH_PREVIEW="$(run_request_preview notional-mismatch "$NOTIONAL_MISMATCH_APPROVAL")"
VALID_PREVIEW="$(run_request_preview valid "$VALID_APPROVAL")"
REUSED_PREVIEW="$(run_request_preview reused "$VALID_APPROVAL")"

python3 - "$PENDING_PREVIEW" "$EXPIRED_PREVIEW" "$REVOKED_PREVIEW" "$USED_PREVIEW" "$RUN_MISMATCH_PREVIEW" "$SYMBOL_MISMATCH_PREVIEW" "$NOTIONAL_MISMATCH_PREVIEW" "$VALID_PREVIEW" "$REUSED_PREVIEW" "$VALID_APPROVAL" <<'PY'
import json
import sys
from pathlib import Path

paths = sys.argv[1:10]
valid_approval_path = Path(sys.argv[10])
names = [
    "pending",
    "expired",
    "revoked",
    "used",
    "run-mismatch",
    "symbol-mismatch",
    "notional-mismatch",
    "valid",
    "reused",
]
artifacts = dict(zip(names, [json.loads(Path(path).read_text()) for path in paths]))

def require(condition, message):
    if not condition:
        raise SystemExit(message)

expected_issues = {
    "pending": "manual_approval_not_approved",
    "expired": "manual_approval_expired",
    "revoked": "manual_approval_revoked",
    "used": "manual_approval_used",
    "run-mismatch": "manual_approval_run_id_mismatch",
    "symbol-mismatch": "manual_approval_symbol_mismatch",
    "notional-mismatch": "manual_approval_notional_mismatch",
    "reused": "manual_approval_used",
}
for name, expected_issue in expected_issues.items():
    artifact = artifacts[name]
    require(artifact["status"] == "blocked_manual_approval_lifecycle", (name, artifact))
    require(artifact["manual_approval_lifecycle_valid"] is False, (name, artifact))
    require(expected_issue in artifact["manual_approval_lifecycle_issues"], (name, artifact))
    require(artifact["request_preview_allowed"] is False, (name, artifact))
    require(artifact["request_preview_built"] is False, (name, artifact))
    require(artifact["request_sent"] is False, (name, artifact))
    require(artifact["production_orders_submitted"] == 0, (name, artifact))
    require(artifact["production_order_mutations_attempted"] == 0, (name, artifact))
    require(artifact["network_attempted"] is False, (name, artifact))

valid = artifacts["valid"]
require(valid["schema_version"] == "ntpro.v150_live_alpha_order_request_preview.v1", valid)
require(valid["status"] == "ready_request_preview_only", valid)
require(valid["manual_approval_lifecycle_status"] == "approval_valid_for_dry_run_request_preview", valid)
require(valid["manual_approval_lifecycle_state"] == "approved", valid)
require(valid["manual_approval_lifecycle_valid"] is True, valid)
require(len(valid["manual_approval_lifecycle_issues"]) == 0, valid)
require(valid["manual_approval_one_time"] is True, valid)
require(valid["manual_approval_used"] is True, valid)
require(valid["manual_approval_consumed"] is True, valid)
require(valid["manual_approval_consume_status"] == "approval_consumed_after_request_preview_created", valid)
require(valid["manual_approval_consume_transition"] == "approved_to_request_preview_created_to_used", valid)
require(valid["request_preview_built"] is True, valid)
require(valid["request_sent"] is False, valid)
require(valid["production_orders_submitted"] == 0, valid)
require(valid["production_order_mutations_attempted"] == 0, valid)
require(valid["network_attempted"] is False, valid)

consumed = json.loads(valid_approval_path.read_text())
require(consumed["approval_state"] == "used", consumed)
require(consumed["approval_used"] is True, consumed)
require(consumed["approval_consumed"] is True, consumed)
require(consumed["request_preview_created"] is True, consumed)
require(consumed["approval_lifecycle_valid"] is False, consumed)
require(consumed["status"] == "approval_consumed_after_request_preview_created", consumed)
PY

if grep -R -q "$SYNTHETIC_API_KEY\|$SYNTHETIC_API_SECRET" "$OUTPUT_DIR"; then
  echo "v15 manual approval lifecycle leaked a synthetic secret into output artifacts" >&2
  exit 1
fi
if grep -R -q "network_attempted\":[[:space:]]*true\|production_orders_submitted\":[[:space:]]*[1-9]" "$OUTPUT_DIR"; then
  echo "v15 manual approval lifecycle recorded forbidden network or order mutation evidence" >&2
  exit 1
fi

echo "v15_manual_approval_lifecycle status=ok root=$APPROVAL_ROOT pending_blocked=true expired_blocked=true revoked_blocked=true used_blocked=true mismatch_blocked=true valid_preview=true production_orders_submitted=0 network_attempted=false"
