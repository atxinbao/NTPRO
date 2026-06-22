#!/usr/bin/env bash
set -euo pipefail

# V150-002: v0.15 production live-alpha order request dry-run builder.
# Safe for local development and CI. It builds redacted request metadata only,
# keeps signatures and signed queries memory-only, opens no network
# connections, calls no execution adapter, and submits no production orders.

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

REQUEST_ROOT="${NTPRO_V15_REQUEST_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v15-request.XXXXXX")}"
OUTPUT_DIR="$REQUEST_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

ORDER_GATE="$OUTPUT_DIR/live-alpha-order-gate.json"
BLOCKED_APPROVAL="$OUTPUT_DIR/blocked-manual-approval-lifecycle.json"
READY_APPROVAL="$OUTPUT_DIR/ready-manual-approval-lifecycle.json"
PRODUCTION_MATERIAL_BLOCKED_APPROVAL="$OUTPUT_DIR/production-material-blocked-manual-approval-lifecycle.json"
BAD_ENDPOINT_APPROVAL="$OUTPUT_DIR/bad-endpoint-manual-approval-lifecycle.json"
BLOCKED_REPORT="$OUTPUT_DIR/blocked-request-preview.json"
READY_REPORT="$OUTPUT_DIR/ready-request-preview.json"
PRODUCTION_MATERIAL_BLOCKED_REPORT="$OUTPUT_DIR/production-material-blocked-request-preview.json"
BLOCKED_STDOUT="$OUTPUT_DIR/blocked.stdout.log"
BLOCKED_STDERR="$OUTPUT_DIR/blocked.stderr.log"
READY_STDOUT="$OUTPUT_DIR/ready.stdout.log"
READY_STDERR="$OUTPUT_DIR/ready.stderr.log"
PRODUCTION_MATERIAL_BLOCKED_STDOUT="$OUTPUT_DIR/production-material-blocked.stdout.log"
PRODUCTION_MATERIAL_BLOCKED_STDERR="$OUTPUT_DIR/production-material-blocked.stderr.log"
BAD_ENDPOINT_STDERR="$OUTPUT_DIR/bad-endpoint.stderr.log"

SYNTHETIC_API_KEY="ntpro_v151003_synthetic_api_key_value"
SYNTHETIC_API_SECRET="ntpro_v151003_synthetic_api_secret_value"

"$NAUTILUS_BIN" live production-live-alpha-dry-run-order-gate \
  --run-id v150-request-preview \
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
  local run_id="$1"
  local output="$2"
  "$NAUTILUS_BIN" live production-live-alpha-manual-approval-lifecycle \
    --run-id "$run_id" \
    --strategy-id ema_cross_btcusdt_v1 \
    --symbol BTCUSDT \
    --notional 10.00 \
    --approval-state approved \
    --manual-approval-id owner-approval-v150-005 \
    --approved-by owner \
    --now-unix-ms 1718400000000 \
    --expires-at-unix-ms 1718400060000 \
    --output "$output" \
    --confirm-dry-run-request-preview-only \
    --confirm-one-time-approval \
    --confirm-no-production-mutation \
    --confirm-dashboard-order-controls-disabled >/dev/null
}

write_approval v150-request-preview-blocked "$BLOCKED_APPROVAL"
write_approval v150-request-preview "$READY_APPROVAL"
write_approval v150-request-preview-production-material-blocked "$PRODUCTION_MATERIAL_BLOCKED_APPROVAL"
write_approval v150-request-preview-bad-endpoint "$BAD_ENDPOINT_APPROVAL"

"$NAUTILUS_BIN" live production-live-alpha-order-request-preview \
  --run-id v150-request-preview-blocked \
  --order-gate "$ORDER_GATE" \
  --manual-approval-lifecycle "$BLOCKED_APPROVAL" \
  --price 10000.00 \
  --timestamp-ms 1718400000000 \
  --api-key-env NTPRO_V150002_API_KEY \
  --api-secret-env NTPRO_V150002_API_SECRET \
  --output "$BLOCKED_REPORT" \
  >"$BLOCKED_STDOUT" \
  2>"$BLOCKED_STDERR"

if [[ -s "$BLOCKED_STDERR" ]]; then
  echo "v15 blocked request preview wrote stderr" >&2
  cat "$BLOCKED_STDERR" >&2
  exit 1
fi
grep -q "status=blocked_endpoint_or_owner_scope" "$BLOCKED_STDOUT"

"$NAUTILUS_BIN" live production-live-alpha-order-request-preview \
  --run-id v150-request-preview \
  --order-gate "$ORDER_GATE" \
  --manual-approval-lifecycle "$READY_APPROVAL" \
  --endpoint-path /api/v3/order \
  --price 10000.00 \
  --time-in-force GTC \
  --timestamp-ms 1718400000000 \
  --recv-window-ms 5000 \
  --api-key-env NTPRO_V150002_API_KEY \
  --api-secret-env NTPRO_V150002_API_SECRET \
  --output "$READY_REPORT" \
  --allow-production-live-alpha-request-preview \
  --confirm-owner-approved-request-preview \
  --confirm-memory-only-signature \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-no-execution-adapter-call \
  --confirm-no-network \
  --confirm-no-listen-key-lifecycle \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-real-funds \
  >"$READY_STDOUT" \
  2>"$READY_STDERR"

if [[ -s "$READY_STDERR" ]]; then
  echo "v15 request preview wrote stderr on pass path" >&2
  cat "$READY_STDERR" >&2
  exit 1
fi
grep -q "live.production_live_alpha_order_request_preview status=ready_request_preview_only" "$READY_STDOUT"
grep -q "request_preview_built=true" "$READY_STDOUT"
grep -q "request_sent=false" "$READY_STDOUT"
grep -q "production_orders_submitted=0" "$READY_STDOUT"
grep -q "production_order_mutations_attempted=0" "$READY_STDOUT"
grep -q "execution_adapter_called=false" "$READY_STDOUT"
grep -q "network_attempted=false" "$READY_STDOUT"
grep -q "dashboard_order_controls_enabled=false" "$READY_STDOUT"

"$NAUTILUS_BIN" live production-live-alpha-order-request-preview \
  --run-id v150-request-preview-production-material-blocked \
  --order-gate "$ORDER_GATE" \
  --manual-approval-lifecycle "$PRODUCTION_MATERIAL_BLOCKED_APPROVAL" \
  --endpoint-path /api/v3/order \
  --price 10000.00 \
  --time-in-force GTC \
  --timestamp-ms 1718400000000 \
  --recv-window-ms 5000 \
  --api-key-env NTPRO_V150002_API_KEY \
  --api-secret-env NTPRO_V150002_API_SECRET \
  --credential-material production_live_alpha \
  --output "$PRODUCTION_MATERIAL_BLOCKED_REPORT" \
  --allow-production-live-alpha-request-preview \
  --confirm-owner-approved-request-preview \
  --confirm-memory-only-signature \
  --confirm-no-production-order-submission \
  --confirm-no-production-order-mutation \
  --confirm-no-execution-adapter-call \
  --confirm-no-network \
  --confirm-no-listen-key-lifecycle \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-real-funds \
  >"$PRODUCTION_MATERIAL_BLOCKED_STDOUT" \
  2>"$PRODUCTION_MATERIAL_BLOCKED_STDERR"

if [[ -s "$PRODUCTION_MATERIAL_BLOCKED_STDERR" ]]; then
  echo "v15 production material blocked preview wrote stderr" >&2
  cat "$PRODUCTION_MATERIAL_BLOCKED_STDERR" >&2
  exit 1
fi
grep -q "status=blocked_endpoint_or_owner_scope" "$PRODUCTION_MATERIAL_BLOCKED_STDOUT"
grep -q "request_preview_built=false" "$PRODUCTION_MATERIAL_BLOCKED_STDOUT"

python3 - "$BLOCKED_REPORT" "$READY_REPORT" "$PRODUCTION_MATERIAL_BLOCKED_REPORT" <<'PY'
import json
import sys
from pathlib import Path

blocked = json.loads(Path(sys.argv[1]).read_text())
ready = json.loads(Path(sys.argv[2]).read_text())
production_material_blocked = json.loads(Path(sys.argv[3]).read_text())

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(blocked["status"] == "blocked_endpoint_or_owner_scope", blocked)
require(blocked["request_preview_built"] is False, blocked)
require(blocked["request_sent"] is False, blocked)
require(blocked["network_attempted"] is False, blocked)
require(blocked["production_orders_submitted"] == 0, blocked)
require(blocked["manual_approval_lifecycle_valid"] is True, blocked)
require(len(blocked["missing_cli_flags"]) == 10, blocked)
require(len(blocked["missing_env_vars"]) == 0, blocked)
require(blocked["credential_material"] == "synthetic", blocked)
require(blocked["production_signing_material_env_read"] is False, blocked)

require(ready["schema_version"] == "ntpro.v150_live_alpha_order_request_preview.v1", ready)
require(ready["status"] == "ready_request_preview_only", ready)
require(ready["endpoint_class"] == "production_mutation_owner_approved_manual_only", ready)
require(ready["endpoint_decision"] == "allow_request_preview_only", ready)
require(ready["request_method"] == "POST", ready)
require(ready["request_target"] == "/api/v3/order", ready)
require(ready["query_shape_without_signature"] == "symbol&side&type&timeInForce&quantity&price&recvWindow&timestamp", ready)
require(ready["signature_preflight"] == "created_in_memory_not_recorded", ready)
require(ready["credential_material"] == "synthetic", ready)
require(ready["production_signing_material_gate_required"] is False, ready)
require(ready["production_signing_material_gate_open"] is False, ready)
require(ready["production_signing_material_env_read"] is False, ready)
require(len(ready["production_signing_material_missing_gate_env_vars"]) == 0, ready)
require(len(ready["missing_env_vars"]) == 0, ready)
require(ready["manual_approval_lifecycle_status"] == "approval_valid_for_dry_run_request_preview", ready)
require(ready["manual_approval_lifecycle_state"] == "approved", ready)
require(ready["manual_approval_lifecycle_valid"] is True, ready)
require(len(ready["manual_approval_lifecycle_issues"]) == 0, ready)
require(ready["manual_approval_one_time"] is True, ready)
require(ready["manual_approval_used"] is True, ready)
require(ready["manual_approval_consumed"] is True, ready)
require(ready["manual_approval_consume_status"] == "approval_consumed_after_request_preview_created", ready)
require(ready["manual_approval_consume_transition"] == "approved_to_request_preview_created_to_used", ready)
for key in [
    "api_key_header_value_recorded",
    "api_secret_value_recorded",
    "signature_recorded",
    "signed_query_recorded",
    "signed_url_recorded",
    "request_body_recorded",
    "raw_request_body_recorded",
    "request_sent",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "production_order_state_reads_allowed",
    "listen_key_lifecycle_allowed",
    "cancel_replace_amend_attempted",
    "order_endpoint_access_attempted",
    "execution_adapter_called",
    "production_adapter_called",
    "matching_engine_submission",
    "dashboard_order_controls_enabled",
    "external_venue_connection",
    "network_attempted",
    "real_orders_submitted",
    "real_funds",
    "production_trading_enabled",
]:
    require(ready[key] is False, (key, ready[key], ready))
for key in [
    "production_order_submissions_attempted",
    "production_orders_submitted",
    "production_order_mutations_attempted",
    "production_order_state_reads_attempted",
    "listen_key_lifecycle_attempted",
    "actual_submission_count",
    "automatic_correction_orders_submitted",
]:
    require(ready[key] == 0, (key, ready[key], ready))
require(ready["request_preview_allowed"] is True, ready)
require(ready["request_preview_built"] is True, ready)
require(ready["signed_request_memory_only"] is True, ready)
require(ready["secrets_redacted"] is True, ready)

require(production_material_blocked["status"] == "blocked_endpoint_or_owner_scope", production_material_blocked)
require(production_material_blocked["credential_material"] == "production_live_alpha", production_material_blocked)
require(production_material_blocked["production_signing_material_gate_required"] is True, production_material_blocked)
require(production_material_blocked["production_signing_material_gate_open"] is False, production_material_blocked)
require(production_material_blocked["production_signing_material_env_read"] is False, production_material_blocked)
require(
    production_material_blocked["production_signing_material_missing_gate_env_vars"]
    == [
        "NTPRO_ALLOW_PRODUCTION_MUTATION_SIGNING_MATERIAL",
        "NTPRO_OWNER_APPROVED_MUTATION_SIGNING_DRY_RUN",
    ],
    production_material_blocked,
)
require(
    production_material_blocked["missing_env_vars"]
    == [
        "NTPRO_ALLOW_PRODUCTION_MUTATION_SIGNING_MATERIAL",
        "NTPRO_OWNER_APPROVED_MUTATION_SIGNING_DRY_RUN",
    ],
    production_material_blocked,
)
require(production_material_blocked["request_preview_built"] is False, production_material_blocked)
require(production_material_blocked["request_sent"] is False, production_material_blocked)
require(production_material_blocked["production_orders_submitted"] == 0, production_material_blocked)
require(production_material_blocked["production_order_mutations_attempted"] == 0, production_material_blocked)
require(production_material_blocked["network_attempted"] is False, production_material_blocked)
PY

if grep -R -q "$SYNTHETIC_API_KEY\|$SYNTHETIC_API_SECRET" "$OUTPUT_DIR"; then
  echo "v15 request preview leaked a synthetic secret into output artifacts" >&2
  exit 1
fi
if grep -R -q "signature=\|signed_query\":[[:space:]]*\"" "$OUTPUT_DIR"; then
  echo "v15 request preview persisted signature or signed query material" >&2
  exit 1
fi
if grep -q "symbol=BTCUSDT" "$READY_REPORT"; then
  echo "v15 request preview persisted raw query values" >&2
  exit 1
fi

set +e
NTPRO_V150002_API_KEY="$SYNTHETIC_API_KEY" \
NTPRO_V150002_API_SECRET="$SYNTHETIC_API_SECRET" \
  "$NAUTILUS_BIN" live production-live-alpha-order-request-preview \
    --run-id v150-request-preview-bad-endpoint \
    --order-gate "$ORDER_GATE" \
    --manual-approval-lifecycle "$BAD_ENDPOINT_APPROVAL" \
    --endpoint-path /api/v3/account \
    --price 10000.00 \
    --timestamp-ms 1718400000000 \
    --api-key-env NTPRO_V150002_API_KEY \
    --api-secret-env NTPRO_V150002_API_SECRET \
    --output "$OUTPUT_DIR/bad-endpoint.json" \
    --allow-production-live-alpha-request-preview \
    --confirm-owner-approved-request-preview \
    --confirm-memory-only-signature \
    --confirm-no-production-order-submission \
    --confirm-no-production-order-mutation \
    --confirm-no-execution-adapter-call \
    --confirm-no-network \
    --confirm-no-listen-key-lifecycle \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-real-funds \
    >/dev/null \
    2>"$BAD_ENDPOINT_STDERR"
bad_endpoint_status=$?
set -e
if [[ "$bad_endpoint_status" -eq 0 ]]; then
  echo "v15 request preview expected bad endpoint to fail closed" >&2
  exit 1
fi
grep -q "allowlist only includes POST /api/v3/order" "$BAD_ENDPOINT_STDERR"

echo "v15_live_order_request_dry_run_builder status=ok root=$REQUEST_ROOT request_sent=false network_attempted=false production_orders_submitted=0 execution_adapter_called=false"
