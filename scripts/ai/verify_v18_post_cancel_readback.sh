#!/usr/bin/env bash
set -euo pipefail

# V180-007: v0.18 post-cancel readback contract.
# This verifier stays local/offline. It proves future post-cancel readback
# stores only redacted metadata, classifies terminal/ambiguous states, and never
# sends a cancel request or performs a network readback.

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

GATE_ROOT="${NTPRO_V18_POST_CANCEL_READBACK_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v18-post-cancel-readback.XXXXXX")}"
REDACTION_ROOT="$GATE_ROOT/cancel-response-redaction"
FIXTURE_DIR="$GATE_ROOT/fixtures"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$FIXTURE_DIR" "$OUTPUT_DIR"

NTPRO_V18_SKIP_BUILD=1 \
  NTPRO_V18_NAUTILUS_BIN="$NAUTILUS_BIN" \
  NTPRO_V18_CANCEL_RESPONSE_REDACTION_ROOT="$REDACTION_ROOT" \
  scripts/ai/verify_v18_cancel_response_redaction.sh >/dev/null

VALID_REDACTION="$REDACTION_ROOT/command-output/cancel-response-redaction-ready.json"
if [[ ! -s "$VALID_REDACTION" ]]; then
  echo "cancel response redaction verifier did not produce $VALID_REDACTION" >&2
  exit 1
fi

python3 - "$FIXTURE_DIR" "$VALID_REDACTION" "$FIXTURE_DIR/invalid-redaction.json" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
valid_redaction = Path(sys.argv[2])
invalid_redaction = Path(sys.argv[3])

def write(path: Path, payload: dict) -> None:
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")

for state in ["CANCELED", "FILLED", "REJECTED", "EXPIRED", "MISSING", "UNKNOWN"]:
    write(root / f"post-cancel-readback-{state.lower()}.json", {
        "symbol": "BTCUSDT",
        "orderId": 123456789,
        "clientOrderId": "owner-approved-v160-single-shot",
        "origClientOrderId": "owner-approved-v160-single-shot",
        "updateTime": 1718400000001,
        "status": state,
    })

write(root / "forbidden-post-cancel-readback.json", {
    "symbol": "BTCUSDT",
    "orderId": 123456789,
    "clientOrderId": "owner-approved-v160-single-shot",
    "origClientOrderId": "owner-approved-v160-single-shot",
    "status": "CANCELED",
    "headers": {"X-MBX-APIKEY": "must_not_persist"},
    "body": {"raw": "raw readback must not persist"},
    "apiSecret": "apiSecret must not persist",
    "payload": {"raw": "unrestricted"},
    "fills": [{"price": "1", "qty": "1"}],
})

payload = json.loads(valid_redaction.read_text())
payload["status"] = "blocked_source_artifact"
payload["response_redaction_ready"] = False
invalid_redaction.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY

run_readback() {
  local name="$1"
  local redaction="$2"
  local readback="$3"
  local all_flags="$4"
  local output="$OUTPUT_DIR/post-cancel-readback-$name.json"
  local cmd=(
    "$NAUTILUS_BIN" live production-mutation-post-cancel-readback
    --run-id v180-production-mutation-post-cancel-readback
    --cancel-response-redaction "$redaction"
    --readback "$readback"
    --output "$output"
  )
  if [[ "$all_flags" == "true" ]]; then
    cmd+=(
      --allow-production-mutation-post-cancel-readback
      --confirm-cancel-response-redaction-ready
      --confirm-readback-metadata-only
      --confirm-terminal-and-ambiguous-classification
      --confirm-no-raw-readback-persistence
      --confirm-no-headers-persistence
      --confirm-no-secret-persistence
      --confirm-no-mutation
      --confirm-no-retry
      --confirm-no-remediation
      --confirm-no-cancel
      --confirm-no-network
      --confirm-dashboard-order-controls-disabled
    )
  fi
  "${cmd[@]}" >/dev/null
  printf '%s\n' "$output"
}

READY_CANCELED="$(run_readback canceled "$VALID_REDACTION" "$FIXTURE_DIR/post-cancel-readback-canceled.json" true)"
READY_FILLED="$(run_readback filled "$VALID_REDACTION" "$FIXTURE_DIR/post-cancel-readback-filled.json" true)"
READY_REJECTED="$(run_readback rejected "$VALID_REDACTION" "$FIXTURE_DIR/post-cancel-readback-rejected.json" true)"
READY_EXPIRED="$(run_readback expired "$VALID_REDACTION" "$FIXTURE_DIR/post-cancel-readback-expired.json" true)"
READY_MISSING="$(run_readback missing "$VALID_REDACTION" "$FIXTURE_DIR/post-cancel-readback-missing.json" true)"
READY_UNKNOWN="$(run_readback unknown "$VALID_REDACTION" "$FIXTURE_DIR/post-cancel-readback-unknown.json" true)"
FORBIDDEN="$(run_readback forbidden "$VALID_REDACTION" "$FIXTURE_DIR/forbidden-post-cancel-readback.json" true)"
MISSING_FLAGS="$(run_readback missing-flags "$VALID_REDACTION" "$FIXTURE_DIR/post-cancel-readback-canceled.json" false)"
INVALID_SOURCE="$(run_readback invalid-source "$FIXTURE_DIR/invalid-redaction.json" "$FIXTURE_DIR/post-cancel-readback-canceled.json" true)"

python3 - "$READY_CANCELED" "$READY_FILLED" "$READY_REJECTED" "$READY_EXPIRED" "$READY_MISSING" "$READY_UNKNOWN" "$FORBIDDEN" "$MISSING_FLAGS" "$INVALID_SOURCE" <<'PY'
import json
import sys
from pathlib import Path

artifacts = [json.loads(Path(path).read_text()) for path in sys.argv[1:]]
ready = artifacts[:6]
forbidden, missing_flags, invalid_source = artifacts[6:]

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
        "raw_exchange_response_recorded",
        "raw_readback_body_recorded",
        "response_body_recorded",
        "response_headers_recorded",
        "unrestricted_payload_recorded",
        "account_balances_recorded",
        "fills_recorded",
        "readback_execution_attempted",
        "order_state_read_attempted",
        "actual_cancel_send_allowed",
        "cancel_attempted",
        "network_attempted",
        "network_readback_endpoint_attempted",
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
    require(artifact["production_order_state_reads_attempted"] == 0, artifact)
    require(artifact["cancel_requests_sent"] == 0, artifact)
    require(artifact["production_order_mutations_attempted"] == 0, artifact)

expected = {
    "CANCELED": ("terminal_canceled", "cancel_confirmed", True, False, True),
    "FILLED": ("terminal_filled", "filled_before_or_during_cancel", True, False, True),
    "REJECTED": ("terminal_rejected", "cancel_or_order_rejected", True, False, True),
    "EXPIRED": ("terminal_expired", "order_expired", True, False, True),
    "MISSING": ("ambiguous_missing", "order_missing_manual_review", False, True, False),
    "UNKNOWN": ("ambiguous_unknown", "unknown_state_manual_review", False, True, True),
}

for artifact in ready:
    state = artifact["readback_state"]
    state_class, outcome, terminal, ambiguous, order_found = expected[state]
    require(artifact["schema_version"] == "ntpro.v180_post_cancel_readback.v1", artifact)
    require(artifact["artifact_type"] == "post_cancel_readback", artifact)
    require(artifact["status"] == "ready_post_cancel_readback_classified", artifact)
    require(artifact["post_cancel_readback_ready"] is True, artifact)
    require(artifact["post_cancel_readback_classified"] is True, artifact)
    require(artifact["redacted_metadata_only"] is True, artifact)
    require(artifact["readback_state_class"] == state_class, artifact)
    require(artifact["readback_outcome"] == outcome, artifact)
    require(artifact["terminal_state_observed"] is terminal, artifact)
    require(artifact["ambiguous_state_observed"] is ambiguous, artifact)
    require(artifact["order_found"] is order_found, artifact)
    require(artifact["order_lineage_preserved"] is True, artifact)
    require(artifact["source_artifact_issues"] == [], artifact)
    require(artifact["missing_cli_flags"] == [], artifact)
    require(artifact["forbidden_readback_markers"] == [], artifact)
    require(artifact["unsupported_readback_states"] == [], artifact)
    require(artifact["readback_order_id"].startswith("readback_order_id:sha256:"), artifact)
    require(artifact["readback_client_order_id"].startswith("readback_client_order_id:sha256:"), artifact)
    require(artifact["readback_orig_client_order_id"].startswith("readback_orig_client_order_id:sha256:"), artifact)
    assert_false_boundary(artifact)

require(forbidden["status"] == "blocked_forbidden_readback_marker", forbidden)
require(forbidden["post_cancel_readback_ready"] is False, forbidden)
require(any("$.headers" in marker for marker in forbidden["forbidden_readback_markers"]), forbidden)
require(any("$.body" in marker for marker in forbidden["forbidden_readback_markers"]), forbidden)
require(any("$.apiSecret" in marker for marker in forbidden["forbidden_readback_markers"]), forbidden)
require(any("$.payload" in marker for marker in forbidden["forbidden_readback_markers"]), forbidden)
require(any("$.fills" in marker for marker in forbidden["forbidden_readback_markers"]), forbidden)
assert_false_boundary(forbidden)

require(missing_flags["status"] == "blocked_missing_gate", missing_flags)
require("--allow-production-mutation-post-cancel-readback" in missing_flags["missing_cli_flags"], missing_flags)
require("--confirm-no-mutation" in missing_flags["missing_cli_flags"], missing_flags)
assert_false_boundary(missing_flags)

require(invalid_source["status"] == "blocked_source_artifact", invalid_source)
require("cancel_response_redaction_status_blocked_source_artifact" in invalid_source["source_artifact_issues"], invalid_source)
require("cancel_response_redaction_response_redaction_ready_not_true" in invalid_source["source_artifact_issues"], invalid_source)
assert_false_boundary(invalid_source)
PY

for output in "$OUTPUT_DIR"/*.json; do
  body="$(cat "$output")"
  for token in "123456789" "owner-approved-v160-single-shot" "X-MBX-APIKEY" "apiSecret must not persist" "raw readback must not persist" "signature=must_not_persist" "signedQuery=" "signedUrl="; do
    if [[ "$body" == *"$token"* ]]; then
      echo "$output contains forbidden token $token" >&2
      exit 1
    fi
  done
done

echo "verify_v18_post_cancel_readback PASS root=$GATE_ROOT states=CANCELED,FILLED,REJECTED,EXPIRED,MISSING,UNKNOWN network_attempted=false cancel_attempted=false readback_execution_attempted=false retry_attempted=false remediation_attempted=false dashboard_cancel_controls_enabled=false"
