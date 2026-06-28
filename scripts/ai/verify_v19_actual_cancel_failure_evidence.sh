#!/usr/bin/env bash
set -euo pipefail

# V190-007: v0.19 actual-cancel failure and partial-success evidence.
# This verifier stays local/offline. It proves failure, recovered, degraded,
# and partial-success outcomes are explicit and do not trigger retry,
# remediation, compensation trades, second cancel, Dashboard controls, or
# network execution.

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

GATE_ROOT="${NTPRO_V19_ACTUAL_CANCEL_FAILURE_EVIDENCE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v19-actual-cancel-failure-evidence.XXXXXX")}"
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
readback("readback-rejected.json", "REJECTED", "0", "1")
readback("readback-timeout.json", "UNKNOWN", "0", "1", readbackResult="timeout")
readback("readback-unknown.json", "UNKNOWN", "0", "1")
readback("readback-partial-fill.json", "PARTIALLY_FILLED", "0.4", "1", remainingQty="0.6")
readback("readback-filled-before-cancel.json", "FILLED", "1", "1")

def ref(path: str, artifact_type: str, status: str, **extra) -> None:
    payload = {
        "schema_version": "ntpro.synthetic_actual_cancel_failure_evidence_ref.v1",
        "artifact_type": artifact_type,
        "status": status,
        "ready": True,
        "order_lineage_id": "lineage-v160-single-shot",
        "symbol": "BTCUSDT",
        "account_label": "prod-account-redacted",
        "venue": "binance_spot",
    }
    payload.update(extra)
    write(root / path, payload)

ref("request-ref.json", "actual_cancel_request_ref", "request_ref_recorded")
ref("response-ref.json", "actual_cancel_response_ref", "response_ref_recorded")
ref("response-ref-venue-unavailable.json", "actual_cancel_response_ref", "response_ref_recorded", venue_unavailable=True)
ref("readback-ref.json", "actual_cancel_readback_ref", "readback_ref_recorded")
ref("audit-ref.json", "actual_cancel_audit_ref", "audit_ref_recorded")
ref("audit-ref-adapter-failure.json", "actual_cancel_audit_ref", "audit_ref_recorded", adapter_failure=True)
write(root / "invalid-ref.json", {})
PY

run_reconciliation() {
  local name="$1"
  local readback="$2"
  local output="$OUTPUT_DIR/reconciliation-$name.json"
  "$NAUTILUS_BIN" live production-mutation-actual-cancel-readback-reconciliation \
    --run-id "v190-actual-cancel-readback-reconciliation-$name" \
    --actual-cancel-attempt "$FIXTURE_DIR/actual-cancel-attempt.json" \
    --readback "$readback" \
    --expected-order-lineage-id lineage-v160-single-shot \
    --expected-symbol BTCUSDT \
    --expected-account-label prod-account-redacted \
    --venue binance_spot \
    --output "$output" \
    --allow-production-mutation-actual-cancel-readback-reconciliation \
    --confirm-actual-cancel-attempt-recorded \
    --confirm-readback-required \
    --confirm-readback-metadata-only \
    --confirm-order-status-reconciled \
    --confirm-execution-fill-status-reconciled \
    --confirm-remaining-quantity-reconciled \
    --confirm-risk-state-recorded \
    --confirm-local-audit-state-recorded \
    --confirm-dashboard-read-only-consumable \
    --confirm-no-raw-readback-persistence \
    --confirm-no-headers-persistence \
    --confirm-no-secret-persistence \
    --confirm-no-retry \
    --confirm-no-remediation \
    --confirm-no-second-cancel \
    --confirm-no-network \
    --confirm-dashboard-order-controls-disabled >/dev/null
  printf '%s\n' "$output"
}

run_failure_evidence() {
  local name="$1"
  local reconciliation="$2"
  local response_ref="$3"
  local audit_ref="$4"
  local output="$OUTPUT_DIR/failure-evidence-$name.json"
  "$NAUTILUS_BIN" live production-mutation-actual-cancel-failure-evidence \
    --run-id "v190-actual-cancel-failure-evidence-$name" \
    --readback-reconciliation "$reconciliation" \
    --request-ref "$FIXTURE_DIR/request-ref.json" \
    --response-ref "$response_ref" \
    --readback-ref "$FIXTURE_DIR/readback-ref.json" \
    --audit-ref "$audit_ref" \
    --expected-order-lineage-id lineage-v160-single-shot \
    --expected-symbol BTCUSDT \
    --expected-account-label prod-account-redacted \
    --venue binance_spot \
    --output "$output" \
    --allow-production-mutation-actual-cancel-failure-evidence \
    --confirm-request-ref-recorded \
    --confirm-response-ref-recorded \
    --confirm-readback-ref-recorded \
    --confirm-audit-ref-recorded \
    --confirm-failure-outcomes-classified \
    --confirm-operator-action-model \
    --confirm-unknown-not-recovered \
    --confirm-partial-fill-residual-risk \
    --confirm-dashboard-release-gate-consumable \
    --confirm-no-retry \
    --confirm-no-remediation \
    --confirm-no-compensation-trade \
    --confirm-no-network \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-secret-persistence >/dev/null
  printf '%s\n' "$output"
}

CONFIRMED_REC="$(run_reconciliation confirmed "$FIXTURE_DIR/readback-confirmed.json")"
ALREADY_REC="$(run_reconciliation already-cancelled "$FIXTURE_DIR/readback-already-cancelled.json")"
REJECTED_REC="$(run_reconciliation rejected "$FIXTURE_DIR/readback-rejected.json")"
TIMEOUT_REC="$(run_reconciliation timeout "$FIXTURE_DIR/readback-timeout.json")"
UNKNOWN_REC="$(run_reconciliation unknown "$FIXTURE_DIR/readback-unknown.json")"
PARTIAL_REC="$(run_reconciliation partial-fill "$FIXTURE_DIR/readback-partial-fill.json")"
FILLED_REC="$(run_reconciliation filled-before-cancel "$FIXTURE_DIR/readback-filled-before-cancel.json")"

CONFIRMED="$(run_failure_evidence confirmed "$CONFIRMED_REC" "$FIXTURE_DIR/response-ref.json" "$FIXTURE_DIR/audit-ref.json")"
ALREADY="$(run_failure_evidence already-cancelled "$ALREADY_REC" "$FIXTURE_DIR/response-ref.json" "$FIXTURE_DIR/audit-ref.json")"
REJECTED="$(run_failure_evidence rejected "$REJECTED_REC" "$FIXTURE_DIR/response-ref.json" "$FIXTURE_DIR/audit-ref.json")"
TIMEOUT="$(run_failure_evidence timeout "$TIMEOUT_REC" "$FIXTURE_DIR/response-ref.json" "$FIXTURE_DIR/audit-ref.json")"
UNKNOWN="$(run_failure_evidence unknown "$UNKNOWN_REC" "$FIXTURE_DIR/response-ref.json" "$FIXTURE_DIR/audit-ref.json")"
PARTIAL="$(run_failure_evidence partial-fill "$PARTIAL_REC" "$FIXTURE_DIR/response-ref.json" "$FIXTURE_DIR/audit-ref.json")"
FILLED="$(run_failure_evidence filled-before-cancel "$FILLED_REC" "$FIXTURE_DIR/response-ref.json" "$FIXTURE_DIR/audit-ref.json")"
VENUE="$(run_failure_evidence venue-unavailable "$CONFIRMED_REC" "$FIXTURE_DIR/response-ref-venue-unavailable.json" "$FIXTURE_DIR/audit-ref.json")"
ADAPTER="$(run_failure_evidence adapter-failure "$CONFIRMED_REC" "$FIXTURE_DIR/response-ref.json" "$FIXTURE_DIR/audit-ref-adapter-failure.json")"

MISSING_FLAGS="$OUTPUT_DIR/failure-evidence-missing-flags.json"
"$NAUTILUS_BIN" live production-mutation-actual-cancel-failure-evidence \
  --run-id v190-actual-cancel-failure-evidence-missing-flags \
  --readback-reconciliation "$CONFIRMED_REC" \
  --request-ref "$FIXTURE_DIR/request-ref.json" \
  --response-ref "$FIXTURE_DIR/response-ref.json" \
  --readback-ref "$FIXTURE_DIR/readback-ref.json" \
  --audit-ref "$FIXTURE_DIR/audit-ref.json" \
  --expected-order-lineage-id lineage-v160-single-shot \
  --expected-symbol BTCUSDT \
  --expected-account-label prod-account-redacted \
  --venue binance_spot \
  --output "$MISSING_FLAGS" >/dev/null

INVALID_REF="$OUTPUT_DIR/failure-evidence-invalid-request-ref.json"
"$NAUTILUS_BIN" live production-mutation-actual-cancel-failure-evidence \
  --run-id v190-actual-cancel-failure-evidence-invalid-ref \
  --readback-reconciliation "$CONFIRMED_REC" \
  --request-ref "$FIXTURE_DIR/invalid-ref.json" \
  --response-ref "$FIXTURE_DIR/response-ref.json" \
  --readback-ref "$FIXTURE_DIR/readback-ref.json" \
  --audit-ref "$FIXTURE_DIR/audit-ref.json" \
  --expected-order-lineage-id lineage-v160-single-shot \
  --expected-symbol BTCUSDT \
  --expected-account-label prod-account-redacted \
  --venue binance_spot \
  --output "$INVALID_REF" \
  --allow-production-mutation-actual-cancel-failure-evidence \
  --confirm-request-ref-recorded \
  --confirm-response-ref-recorded \
  --confirm-readback-ref-recorded \
  --confirm-audit-ref-recorded \
  --confirm-failure-outcomes-classified \
  --confirm-operator-action-model \
  --confirm-unknown-not-recovered \
  --confirm-partial-fill-residual-risk \
  --confirm-dashboard-release-gate-consumable \
  --confirm-no-retry \
  --confirm-no-remediation \
  --confirm-no-compensation-trade \
  --confirm-no-network \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-secret-persistence >/dev/null

if "$NAUTILUS_BIN" live production-mutation-actual-cancel-failure-evidence \
  --run-id v190-actual-cancel-failure-evidence-missing-path \
  --readback-reconciliation "$CONFIRMED_REC" \
  --request-ref "$FIXTURE_DIR/does-not-exist.json" \
  --response-ref "$FIXTURE_DIR/response-ref.json" \
  --readback-ref "$FIXTURE_DIR/readback-ref.json" \
  --audit-ref "$FIXTURE_DIR/audit-ref.json" \
  --expected-order-lineage-id lineage-v160-single-shot \
  --expected-symbol BTCUSDT \
  --expected-account-label prod-account-redacted \
  --venue binance_spot \
  --output "$OUTPUT_DIR/failure-evidence-missing-path.json" \
  --allow-production-mutation-actual-cancel-failure-evidence \
  --confirm-request-ref-recorded \
  --confirm-response-ref-recorded \
  --confirm-readback-ref-recorded \
  --confirm-audit-ref-recorded \
  --confirm-failure-outcomes-classified \
  --confirm-operator-action-model \
  --confirm-unknown-not-recovered \
  --confirm-partial-fill-residual-risk \
  --confirm-dashboard-release-gate-consumable \
  --confirm-no-retry \
  --confirm-no-remediation \
  --confirm-no-compensation-trade \
  --confirm-no-network \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-secret-persistence >/dev/null 2>&1; then
  echo "missing request ref unexpectedly succeeded" >&2
  exit 1
fi

python3 - "$CONFIRMED" "$ALREADY" "$REJECTED" "$TIMEOUT" "$UNKNOWN" "$PARTIAL" "$FILLED" "$VENUE" "$ADAPTER" "$MISSING_FLAGS" "$INVALID_REF" <<'PY'
import json
import sys
from pathlib import Path

confirmed, already, rejected, timeout, unknown, partial, filled, venue, adapter, missing_flags, invalid_ref = [
    json.loads(Path(path).read_text()) for path in sys.argv[1:]
]

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
        "compensation_trade_attempted",
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
    (confirmed, "ready_actual_cancel_failure_recovered_cancel_confirmed", "cancel_confirmed", "recovered", True, False, False, False, False),
    (already, "ready_actual_cancel_failure_recovered_already_cancelled", "already_cancelled", "recovered", True, False, False, False, False),
    (rejected, "ready_actual_cancel_failure_rejected", "rejected", "failed", False, True, False, True, True),
    (timeout, "ready_actual_cancel_failure_timeout", "timeout", "failed", False, True, False, True, True),
    (unknown, "ready_actual_cancel_failure_unknown", "unknown", "failed", False, True, False, True, True),
    (partial, "ready_actual_cancel_partial_success_partial_fill", "partial_fill", "partial_success", False, False, True, True, True),
    (filled, "ready_actual_cancel_partial_success_filled_before_cancel", "filled_before_cancel", "partial_success", False, False, True, True, True),
    (venue, "ready_actual_cancel_failure_venue_unavailable", "venue_unavailable", "failed", False, True, False, True, True),
    (adapter, "ready_actual_cancel_failure_adapter_failure", "adapter_failure", "failed", False, True, False, True, True),
]

for artifact, status, outcome, category, recovered, failed, partial_success, residual_visible, action_required in expected:
    require(artifact["schema_version"] == "ntpro.v190_actual_cancel_failure_evidence.v1", artifact)
    require(artifact["artifact_type"] == "actual_cancel_failure_evidence", artifact)
    require(artifact["status"] == status, artifact)
    require(artifact["evidence_ready"] is True, artifact)
    require(artifact["failure_evidence_ready"] is True, artifact)
    require(artifact["dashboard_read_only_consumable"] is True, artifact)
    require(artifact["release_gate_consumable"] is True, artifact)
    require(artifact["request_response_readback_audit_refs_recorded"] is True, artifact)
    require(artifact["cancel_outcome"] == outcome, artifact)
    require(artifact["outcome_category"] == category, artifact)
    require(artifact["recovered"] is recovered, artifact)
    require(artifact["failed"] is failed, artifact)
    require(artifact["partial_success"] is partial_success, artifact)
    require(artifact["residual_risk_visible"] is residual_visible, artifact)
    require(artifact["operator_action_required"] is action_required, artifact)
    require(artifact["unknown_not_recovered"] is True, artifact)
    require(artifact["source_artifact_issues"] == [], artifact)
    require(artifact["lineage_issues"] == [], artifact)
    require(artifact["missing_cli_flags"] == [], artifact)
    assert_false_boundary(artifact)

require(unknown["recovered"] is False, unknown)
require(unknown["actual_cancel_followup_complete"] is False, unknown)
require(partial["partial_fill_residual_risk_visible"] is True, partial)
require(partial["residual_risk_state"] == "partial_fill_residual_risk_manual_review", partial)

require(missing_flags["status"] == "blocked_missing_gate", missing_flags)
require(missing_flags["evidence_ready"] is False, missing_flags)
require("--allow-production-mutation-actual-cancel-failure-evidence" in missing_flags["missing_cli_flags"], missing_flags)
require("--confirm-request-ref-recorded" in missing_flags["missing_cli_flags"], missing_flags)
require("--confirm-no-compensation-trade" in missing_flags["missing_cli_flags"], missing_flags)
assert_false_boundary(missing_flags)

require(invalid_ref["status"] == "blocked_source_artifact", invalid_ref)
require(invalid_ref["evidence_ready"] is False, invalid_ref)
require("request_ref_missing_artifact_type" in invalid_ref["source_artifact_issues"], invalid_ref)
require("request_ref_missing_status" in invalid_ref["source_artifact_issues"], invalid_ref)
assert_false_boundary(invalid_ref)
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

echo "verify_v19_actual_cancel_failure_evidence PASS root=$GATE_ROOT outcomes=cancel_confirmed,already_cancelled,rejected,timeout,unknown,partial_fill,filled_before_cancel,venue_unavailable,adapter_failure unknown_not_recovered=true partial_fill_residual_risk_visible=true network_attempted=false cancel_attempted=false retry_attempted=false remediation_attempted=false compensation_trade_attempted=false dashboard_cancel_controls_enabled=false"
