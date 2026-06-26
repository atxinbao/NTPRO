#!/usr/bin/env bash
set -euo pipefail

# V180-006: v0.18 cancel response redaction contract.
# This verifier stays local/offline. It proves future cancel response handling
# stores only redacted metadata and never sends a cancel request.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh
export NTPRO_SOURCE_COMMIT="${NTPRO_SOURCE_COMMIT:-$(git rev-parse HEAD)}"
export NTPRO_SOURCE_RELEASE_TAG="${NTPRO_SOURCE_RELEASE_TAG:-unreleased-v18-local-gate}"

if [[ "${NTPRO_V18_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V18_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V18_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

GATE_ROOT="${NTPRO_V18_CANCEL_RESPONSE_REDACTION_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v18-cancel-response-redaction.XXXXXX")}"
MANUAL_ROOT="$GATE_ROOT/manual-owner-approval"
FIXTURE_DIR="$GATE_ROOT/fixtures"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$FIXTURE_DIR" "$OUTPUT_DIR"

NTPRO_V18_SKIP_BUILD=1 \
  NTPRO_V18_NAUTILUS_BIN="$NAUTILUS_BIN" \
  NTPRO_V18_MANUAL_OWNER_APPROVAL_ROOT="$MANUAL_ROOT" \
  scripts/ai/verify_v18_manual_owner_approval_lifecycle.sh >/dev/null

VALID_APPROVAL="$MANUAL_ROOT/command-output/manual-owner-approval-valid.json"
if [[ ! -s "$VALID_APPROVAL" ]]; then
  echo "manual owner approval verifier did not produce $VALID_APPROVAL" >&2
  exit 1
fi

python3 - "$FIXTURE_DIR" "$VALID_APPROVAL" "$FIXTURE_DIR/consumed-approval.json" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
valid_approval = Path(sys.argv[2])
consumed_approval = Path(sys.argv[3])

(root / "safe-cancel-response.json").write_text(json.dumps({
    "symbol": "BTCUSDT",
    "orderId": 123456789,
    "clientOrderId": "owner-approved-v160-single-shot",
    "origClientOrderId": "owner-approved-v160-single-shot",
    "transactTime": 1718400000000,
    "status": "CANCELED",
}, indent=2, sort_keys=True) + "\n")

(root / "forbidden-cancel-response.json").write_text(json.dumps({
    "symbol": "BTCUSDT",
    "orderId": 123456789,
    "clientOrderId": "owner-approved-v160-single-shot",
    "origClientOrderId": "owner-approved-v160-single-shot",
    "status": "CANCELED",
    "headers": {"redacted": "present"},
    "body": {"raw": "raw response must not persist"},
    "signature": "signature=must_not_persist",
    "payload": {"raw": "unrestricted"},
}, indent=2, sort_keys=True) + "\n")

payload = json.loads(valid_approval.read_text())
payload["approval_consumed"] = True
consumed_approval.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY

run_redaction() {
  local name="$1"
  local approval="$2"
  local response="$3"
  local all_flags="$4"
  local output="$OUTPUT_DIR/cancel-response-redaction-$name.json"
  local cmd=(
    "$NAUTILUS_BIN" live production-mutation-cancel-response-redaction
    --run-id v180-production-mutation-cancel-response-redaction
    --manual-owner-approval-lifecycle "$approval"
    --response "$response"
    --output "$output"
  )
  if [[ "$all_flags" == "true" ]]; then
    cmd+=(
      --allow-production-mutation-cancel-response-redaction
      --confirm-manual-owner-approval-lifecycle-ready
      --confirm-no-raw-response-persistence
      --confirm-no-headers-persistence
      --confirm-no-secret-persistence
      --confirm-cancel-metadata-only
      --confirm-no-account-balances
      --confirm-no-unrestricted-payload
      --confirm-no-retry
      --confirm-no-cancel
      --confirm-no-network
      --confirm-dashboard-order-controls-disabled
    )
  fi
  "${cmd[@]}" >/dev/null
  printf '%s\n' "$output"
}

READY="$(run_redaction ready "$VALID_APPROVAL" "$FIXTURE_DIR/safe-cancel-response.json" true)"
FORBIDDEN="$(run_redaction forbidden "$VALID_APPROVAL" "$FIXTURE_DIR/forbidden-cancel-response.json" true)"
MISSING="$(run_redaction missing "$VALID_APPROVAL" "$FIXTURE_DIR/safe-cancel-response.json" false)"
CONSUMED="$(run_redaction consumed "$FIXTURE_DIR/consumed-approval.json" "$FIXTURE_DIR/safe-cancel-response.json" true)"

python3 - "$READY" "$FORBIDDEN" "$MISSING" "$CONSUMED" <<'PY'
import json
import sys
from pathlib import Path

ready, forbidden, missing, consumed = [json.loads(Path(path).read_text()) for path in sys.argv[1:]]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def assert_false_boundary(artifact):
    for field in [
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "api_key_header_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "request_body_recorded",
        "raw_request_body_recorded",
        "raw_exchange_response_recorded",
        "response_body_recorded",
        "response_headers_recorded",
        "unrestricted_payload_recorded",
        "account_balances_recorded",
        "fills_recorded",
        "actual_cancel_send_allowed",
        "cancel_attempted",
        "network_attempted",
        "network_cancel_endpoint_attempted",
        "retry_attempted",
        "replace_attempted",
        "amend_attempted",
        "flatten_attempted",
        "remediation_attempted",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "production_order_mutation_allowed",
        "dashboard_order_controls_enabled",
        "dashboard_cancel_controls_enabled",
    ]:
        require(artifact[field] is False, (field, artifact))
    require(artifact["cancel_requests_sent"] == 0, artifact)

require(ready["schema_version"] == "ntpro.v180_cancel_response_redaction.v1", ready)
require(ready["artifact_type"] == "cancel_response_redaction", ready)
require(ready["status"] == "ready_cancel_response_redacted", ready)
require(ready["response_redaction_ready"] is True, ready)
require(ready["cancel_response_redacted"] is True, ready)
require(ready["response_shape_validated"] is True, ready)
require(ready["approval_lifecycle_valid"] is True, ready)
require(ready["approval_state"] == "approved", ready)
require(ready["manual_approval_recorded"] is True, ready)
require(ready["approval_consumed"] is False, ready)
require(ready["symbol"] == "BTCUSDT", ready)
require(ready["account_label"] == "prod-account-redacted", ready)
require(ready["exchange_status"] == "CANCELED", ready)
require(ready["cancel_order_id"].startswith("cancel_order_id:sha256:"), ready)
require(ready["cancel_client_order_id"].startswith("cancel_client_order_id:sha256:"), ready)
require(ready["orig_client_order_id"].startswith("orig_client_order_id:sha256:"), ready)
require(ready["source_artifact_issues"] == [], ready)
require(ready["missing_cli_flags"] == [], ready)
require(ready["forbidden_response_markers"] == [], ready)
assert_false_boundary(ready)

require(forbidden["status"] == "blocked_forbidden_response_marker", forbidden)
require(forbidden["response_redaction_ready"] is False, forbidden)
require(any("$.headers" in marker for marker in forbidden["forbidden_response_markers"]), forbidden)
require(any("$.body" in marker for marker in forbidden["forbidden_response_markers"]), forbidden)
require(any("$.signature" in marker for marker in forbidden["forbidden_response_markers"]), forbidden)
assert_false_boundary(forbidden)

require(missing["status"] == "blocked_missing_gate", missing)
require(missing["response_redaction_ready"] is False, missing)
require("--allow-production-mutation-cancel-response-redaction" in missing["missing_cli_flags"], missing)
require("--confirm-no-raw-response-persistence" in missing["missing_cli_flags"], missing)
assert_false_boundary(missing)

require(consumed["status"] == "blocked_source_artifact", consumed)
require(consumed["response_redaction_ready"] is False, consumed)
require(
    "manual_owner_approval_lifecycle_approval_consumed_true" in consumed["source_artifact_issues"],
    consumed,
)
assert_false_boundary(consumed)
PY

for output in "$OUTPUT_DIR"/*.json; do
  body="$(cat "$output")"
  for token in "123456789" "owner-approved-v160-single-shot" "X-MBX-APIKEY" "signature=must_not_persist" "apiSecret" "signedQuery=" "signedUrl=" "raw response must not persist"; do
    if [[ "$body" == *"$token"* ]]; then
      echo "$output contains forbidden token $token" >&2
      exit 1
    fi
  done
done

echo "verify_v18_cancel_response_redaction PASS root=$GATE_ROOT"
