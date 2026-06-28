#!/usr/bin/env bash
set -euo pipefail

# V190-006: v0.19 actual-cancel post-readback reconciliation.
# This verifier stays local/offline. It proves a recorded actual cancel attempt
# cannot be closed without redacted readback reconciliation evidence, and that
# unknown, timeout, already-cancelled, and partial-fill paths remain explicit.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh
export NTPRO_SOURCE_COMMIT="${NTPRO_SOURCE_COMMIT:-$(git rev-parse HEAD)}"
export NTPRO_SOURCE_RELEASE_TAG="${NTPRO_SOURCE_RELEASE_TAG:-unreleased-v19-local-gate}"

if [[ "${NTPRO_V19_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V19_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V19_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

GATE_ROOT="${NTPRO_V19_POST_CANCEL_READBACK_RECONCILIATION_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v19-post-cancel-readback-reconciliation.XXXXXX")}"
FIXTURE_DIR="$GATE_ROOT/fixtures"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$FIXTURE_DIR" "$OUTPUT_DIR"

python3 - "$FIXTURE_DIR" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])

def redacted(kind: str, raw: str) -> str:
    digest = hashlib.sha256(raw.strip().encode()).hexdigest()[:16]
    return f"{kind}:sha256:{digest}:len={len(raw.strip())}"

def write(path: Path, payload: dict) -> None:
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")

write(root / "actual-cancel-attempt.json", {
    "schema_version": "ntpro.v190_actual_cancel_single_shot.v1",
    "run_id": "v190-actual-cancel-single-shot",
    "order_lineage_id": "lineage-v160-single-shot",
    "artifact_type": "actual_cancel_single_shot",
    "status": "actual_cancel_attempt_recorded",
    "actual_cancel_command_ready": True,
    "single_shot_cancel_allowed": True,
    "owner_approval_ready": True,
    "risk_gate_ready": True,
    "release_provenance_ready": True,
    "adapter_boundary_ready": True,
    "adapter_capability_ready": True,
    "approval_consumed_before_send": True,
    "approval_consumed_after_send": True,
    "request_sent": True,
    "cancel_attempted": True,
    "cancel_requests_sent": 1,
    "production_order_mutations_attempted": 1,
    "network_attempted": True,
    "network_cancel_endpoint_attempted": True,
    "http_send_attempted": True,
    "venue_ack_observed": True,
    "readback_required": True,
    "readback_requirement": "post_cancel_readback_required_before_any_retry_or_followup",
    "request_id": "actual-cancel:v190-actual-cancel-single-shot:1718400000000",
    "venue": "binance_spot",
    "symbol": "BTCUSDT",
    "account_label": "prod-account-redacted",
    "known_order_id": redacted("order_id", "123456789"),
    "known_client_order_id": redacted("client_order_id", "owner-approved-v160-single-shot"),
    "cancel_order_identifier_ref": redacted("order_id", "123456789"),
    "source_artifact_issues": [],
    "adapter_capability_issues": [],
    "safety_contract_issues": [],
    "release_manifest_issues": [],
    "missing_cli_flags": [],
    "missing_env_vars": [],
    "retry_attempted": False,
    "replace_attempted": False,
    "amend_attempted": False,
    "flatten_attempted": False,
    "remediation_attempted": False,
    "automatic_cancel_allowed": False,
    "automatic_remediation_allowed": False,
    "bulk_cancel_allowed": False,
    "cancel_all_allowed": False,
    "multi_account_cancel_allowed": False,
    "multi_strategy_cancel_allowed": False,
    "multi_venue_cancel_allowed": False,
    "dashboard_order_controls_enabled": False,
    "dashboard_cancel_controls_enabled": False,
    "dashboard_execution_allowed": False,
    "api_key_value_recorded": False,
    "api_secret_value_recorded": False,
    "api_key_header_value_recorded": False,
    "signature_recorded": False,
    "signed_query_recorded": False,
    "signed_url_recorded": False,
    "request_body_recorded": False,
    "raw_request_body_recorded": False,
    "raw_exchange_response_recorded": False,
    "response_body_recorded": False,
    "response_headers_recorded": False,
})

invalid = json.loads((root / "actual-cancel-attempt.json").read_text())
invalid["status"] = "ready_actual_cancel_command_offline_no_send"
invalid["single_shot_cancel_allowed"] = False
invalid["request_sent"] = False
invalid["cancel_attempted"] = False
invalid["cancel_requests_sent"] = 0
invalid["production_order_mutations_attempted"] = 0
invalid["http_send_attempted"] = False
invalid["readback_required"] = False
write(root / "actual-cancel-no-attempt.json", invalid)

def readback(path: str, status: str, executed: str, orig: str, **extra) -> None:
    payload = {
        "symbol": "BTCUSDT",
        "orderId": 123456789,
        "clientOrderId": "owner-approved-v160-single-shot",
        "origClientOrderId": "owner-approved-v160-single-shot",
        "updateTime": 1718400000001,
        "status": status,
        "executedQty": executed,
        "origQty": orig,
        "remainingQty": "0",
        "localAuditState": "actual_cancel_attempt_recorded",
    }
    payload.update(extra)
    write(root / path, payload)

readback("readback-confirmed.json", "CANCELED", "0", "1")
readback("readback-already-cancelled.json", "CANCELED", "0", "1", alreadyCancelled=True, readbackResult="already_cancelled")
readback("readback-filled-before-cancel.json", "FILLED", "1", "1")
readback("readback-unknown.json", "UNKNOWN", "0", "1")
readback("readback-timeout.json", "UNKNOWN", "0", "1", readbackResult="timeout")
readback("readback-partial-fill.json", "PARTIALLY_FILLED", "0.4", "1", remainingQty="0.6")

write(root / "forbidden-readback.json", {
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
PY

run_reconciliation() {
  local name="$1"
  local actual_cancel="$2"
  local readback="$3"
  local all_flags="$4"
  local output="$OUTPUT_DIR/reconciliation-$name.json"
  local cmd=(
    "$NAUTILUS_BIN" live production-mutation-actual-cancel-readback-reconciliation
    --run-id v190-actual-cancel-readback-reconciliation
    --actual-cancel-attempt "$actual_cancel"
    --readback "$readback"
    --expected-order-lineage-id lineage-v160-single-shot
    --expected-symbol BTCUSDT
    --expected-account-label prod-account-redacted
    --venue binance_spot
    --output "$output"
  )
  if [[ "$all_flags" == "true" ]]; then
    cmd+=(
      --allow-production-mutation-actual-cancel-readback-reconciliation
      --confirm-actual-cancel-attempt-recorded
      --confirm-readback-required
      --confirm-readback-metadata-only
      --confirm-order-status-reconciled
      --confirm-execution-fill-status-reconciled
      --confirm-remaining-quantity-reconciled
      --confirm-risk-state-recorded
      --confirm-local-audit-state-recorded
      --confirm-dashboard-read-only-consumable
      --confirm-no-raw-readback-persistence
      --confirm-no-headers-persistence
      --confirm-no-secret-persistence
      --confirm-no-retry
      --confirm-no-remediation
      --confirm-no-second-cancel
      --confirm-no-network
      --confirm-dashboard-order-controls-disabled
    )
  fi
  "${cmd[@]}" >/dev/null
  printf '%s\n' "$output"
}

ATTEMPT="$FIXTURE_DIR/actual-cancel-attempt.json"
CONFIRMED="$(run_reconciliation confirmed "$ATTEMPT" "$FIXTURE_DIR/readback-confirmed.json" true)"
ALREADY="$(run_reconciliation already-cancelled "$ATTEMPT" "$FIXTURE_DIR/readback-already-cancelled.json" true)"
FILLED="$(run_reconciliation filled-before-cancel "$ATTEMPT" "$FIXTURE_DIR/readback-filled-before-cancel.json" true)"
UNKNOWN="$(run_reconciliation unknown "$ATTEMPT" "$FIXTURE_DIR/readback-unknown.json" true)"
TIMEOUT="$(run_reconciliation timeout "$ATTEMPT" "$FIXTURE_DIR/readback-timeout.json" true)"
PARTIAL="$(run_reconciliation partial-fill "$ATTEMPT" "$FIXTURE_DIR/readback-partial-fill.json" true)"
FORBIDDEN="$(run_reconciliation forbidden "$ATTEMPT" "$FIXTURE_DIR/forbidden-readback.json" true)"
MISSING_FLAGS="$(run_reconciliation missing-flags "$ATTEMPT" "$FIXTURE_DIR/readback-confirmed.json" false)"
INVALID_SOURCE="$(run_reconciliation invalid-source "$FIXTURE_DIR/actual-cancel-no-attempt.json" "$FIXTURE_DIR/readback-confirmed.json" true)"

python3 - "$CONFIRMED" "$ALREADY" "$FILLED" "$UNKNOWN" "$TIMEOUT" "$PARTIAL" "$FORBIDDEN" "$MISSING_FLAGS" "$INVALID_SOURCE" <<'PY'
import json
import sys
from pathlib import Path

artifacts = [json.loads(Path(path).read_text()) for path in sys.argv[1:]]
confirmed, already, filled, unknown, timeout, partial, forbidden, missing_flags, invalid_source = artifacts

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
        "second_cancel_attempted",
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

expected = [
    (confirmed, "ready_actual_cancel_readback_cancel_confirmed", "cancel_confirmed", "ready_cancel_confirmed", False, False, True),
    (already, "ready_actual_cancel_readback_already_cancelled", "already_cancelled", "ready_already_cancelled", False, False, True),
    (filled, "ready_actual_cancel_readback_filled_before_cancel", "filled_before_cancel", "ready_filled_before_cancel", False, True, False),
    (unknown, "degraded_actual_cancel_readback_unknown", "unknown", "degraded_unknown", True, False, False),
    (timeout, "degraded_actual_cancel_readback_timeout", "timeout", "degraded_timeout", True, False, False),
    (partial, "degraded_actual_cancel_readback_inconsistent", "inconsistent", "degraded_inconsistent", True, False, False),
]

for artifact, status, result, reconciliation_status, degraded, filled_before_cancel, followup_complete in expected:
    require(artifact["schema_version"] == "ntpro.v190_actual_cancel_readback_reconciliation.v1", artifact)
    require(artifact["artifact_type"] == "actual_cancel_readback_reconciliation", artifact)
    require(artifact["status"] == status, artifact)
    require(artifact["readback_result"] == result, artifact)
    require(artifact["reconciliation_status"] == reconciliation_status, artifact)
    require(artifact["reconciliation_ready"] is True, artifact)
    require(artifact["readback_evidence_present"] is True, artifact)
    require(artifact["reconciliation_evidence_present"] is True, artifact)
    require(artifact["dashboard_read_only_consumable"] is True, artifact)
    require(artifact["dashboard_audit_view_ready"] is True, artifact)
    require(artifact["degraded"] is degraded, artifact)
    require(artifact["error_state"] is degraded, artifact)
    require(artifact["filled_before_cancel_observed"] is filled_before_cancel, artifact)
    require(artifact["actual_cancel_followup_complete"] is followup_complete, artifact)
    require(artifact["actual_cancel_attempt_recorded"] is True, artifact)
    require(artifact["actual_cancel_request_sent"] is True, artifact)
    require(artifact["readback_required"] is True, artifact)
    require(artifact["source_artifact_issues"] == [], artifact)
    require(artifact["readback_lineage_issues"] == [], artifact)
    require(artifact["forbidden_readback_markers"] == [], artifact)
    require(artifact["unsupported_readback_states"] == [], artifact)
    require(artifact["missing_cli_flags"] == [], artifact)
    require(artifact["readback_order_id"].startswith("readback_order_id:sha256:"), artifact)
    assert_false_boundary(artifact)

require(partial["partial_fill_observed"] is True, partial)
require(partial["inconsistent_observed"] is True, partial)
require(already["already_cancelled_observed"] is True, already)
require(timeout["timeout_observed"] is True, timeout)
require(unknown["unknown_observed"] is True, unknown)

require(forbidden["status"] == "blocked_forbidden_readback_marker", forbidden)
require(forbidden["reconciliation_ready"] is False, forbidden)
require(any("$.headers" in marker for marker in forbidden["forbidden_readback_markers"]), forbidden)
require(any("$.fills" in marker for marker in forbidden["forbidden_readback_markers"]), forbidden)
assert_false_boundary(forbidden)

require(missing_flags["status"] == "blocked_missing_gate", missing_flags)
require(missing_flags["reconciliation_ready"] is False, missing_flags)
require("--allow-production-mutation-actual-cancel-readback-reconciliation" in missing_flags["missing_cli_flags"], missing_flags)
require("--confirm-no-second-cancel" in missing_flags["missing_cli_flags"], missing_flags)
assert_false_boundary(missing_flags)

require(invalid_source["status"] == "blocked_source_artifact", invalid_source)
require(invalid_source["reconciliation_ready"] is False, invalid_source)
require("actual_cancel_attempt_status_ready_actual_cancel_command_offline_no_send" in invalid_source["source_artifact_issues"], invalid_source)
require("actual_cancel_attempt_request_sent_not_true" in invalid_source["source_artifact_issues"], invalid_source)
assert_false_boundary(invalid_source)
PY

for output in "$OUTPUT_DIR"/*.json; do
  body="$(cat "$output")"
  for token in "123456789" "owner-approved-v160-single-shot" "X-MBX-APIKEY" "apiSecret must not persist" "raw readback must not persist" "signature=" "signedQuery=" "signedUrl="; do
    if [[ "$body" == *"$token"* ]]; then
      echo "$output contains forbidden token $token" >&2
      exit 1
    fi
  done
done

echo "verify_v19_post_cancel_readback_reconciliation PASS root=$GATE_ROOT results=cancel_confirmed,already_cancelled,filled_before_cancel,unknown,timeout,inconsistent partial_fill_observed=true network_attempted=false cancel_attempted=false second_cancel_attempted=false retry_attempted=false remediation_attempted=false dashboard_cancel_controls_enabled=false"
