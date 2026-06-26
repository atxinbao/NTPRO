#!/usr/bin/env bash
set -euo pipefail

# V180-004: v0.18 cancel risk gate artifact.
# This verifier stays local/offline. It proves the gate only becomes ready for
# one ready cancel preview whose lineage, symbol, account label, known order
# identifier, orphan risk, risk halt, and owner-approval requirement all match.

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

GATE_ROOT="${NTPRO_V18_CANCEL_RISK_GATE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v18-cancel-risk-gate.XXXXXX")}"
FIXTURE_DIR="$GATE_ROOT/fixtures"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$FIXTURE_DIR" "$OUTPUT_DIR"

python3 - "$FIXTURE_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])

def write(path, payload):
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")

payload = {
    "schema_version": "ntpro.v170_production_mutation_exchange_readback_mapper.v1",
    "run_id": "v170-exchange-readback-mapper-open-orphan",
    "order_lineage_id": "lineage-v160-single-shot",
    "artifact_type": "production_mutation_exchange_readback_mapper",
    "status": "ready_exchange_readback_mapped",
    "mode": "single_mutation_candidate_exchange_readback_mapper",
    "capability": "Production Reconciliation And Orphan Recovery Evidence",
    "capability_expansion_from_v16": "reconciliation_evidence_only",
    "lineage_scope": "single_v16_mutation_candidate",
    "default_fail_closed": True,
    "owner_gated_readback_required": True,
    "local_ledger_ready": True,
    "exchange_readback_mapped": True,
    "reconciliation_classified": False,
    "orphan_risk_detected": False,
    "known_order_id": "123456789",
    "known_client_order_id": "owner-approved-v160-single-shot",
    "symbol": "BTCUSDT",
    "exchange_order_status": "NEW",
    "exchange_order_state": "open",
    "open_order_observed": True,
    "terminal_state_observed": False,
    "order_found": True,
    "open_orders_count": 1,
    "source_artifact_issues": [],
    "malformed_readback_issues": [],
    "missing_cli_flags": [],
    "manual_review_required": False,
    "new_orders_blocked": False,
    "request_sent": True,
}
for field in [
    "network_attempted",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "production_order_state_reads_allowed",
    "listen_key_lifecycle_allowed",
    "duplicate_submit_attempted",
    "retry_attempted",
    "cancel_attempted",
    "replace_attempted",
    "amend_attempted",
    "flatten_attempted",
    "remediation_attempted",
    "automatic_cancel_allowed",
    "automatic_remediation_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "api_key_value_recorded",
    "api_secret_value_recorded",
    "api_key_header_value_recorded",
    "signature_recorded",
    "signed_query_recorded",
    "signed_url_recorded",
    "raw_exchange_response_recorded",
    "response_body_recorded",
    "response_headers_recorded",
]:
    payload[field] = False
write(root / "open-orphan-mapper.json", payload)
PY

run_classifier() {
  "$NAUTILUS_BIN" live production-mutation-reconciliation-classifier \
    --run-id v170-production-mutation-reconciliation-classifier \
    --exchange-readback-mapper "$1" \
    --output "$2" \
    --allow-production-mutation-reconciliation-classifier \
    --confirm-single-v16-mutation-candidate-lineage \
    --confirm-read-only-reconciliation-scope \
    --confirm-no-retry \
    --confirm-no-cancel \
    --confirm-no-remediation \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-secret-persistence
}

run_orphan_detector() {
  "$NAUTILUS_BIN" live production-mutation-orphan-order-detector \
    --run-id v170-production-mutation-orphan-order-detector \
    --reconciliation-classifier "$1" \
    --output "$2" \
    --allow-production-mutation-orphan-order-detector \
    --confirm-single-v16-mutation-candidate-lineage \
    --confirm-read-only-reconciliation-scope \
    --confirm-no-retry \
    --confirm-no-cancel \
    --confirm-no-remediation \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-secret-persistence
}

run_preview() {
  "$NAUTILUS_BIN" live production-mutation-cancel-request-preview \
    --run-id v180-production-mutation-cancel-request-preview \
    --orphan-order-detector "$1" \
    --account-label prod-account-redacted \
    --output "$2" \
    --allow-production-mutation-cancel-request-preview \
    --confirm-single-v16-mutation-candidate-lineage \
    --confirm-orphan-risk-halted \
    --confirm-manual-review-required \
    --confirm-known-order-identifier-only \
    --confirm-no-retry \
    --confirm-no-cancel \
    --confirm-no-network \
    --confirm-no-remediation \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-secret-persistence
}

run_gate() {
  "$NAUTILUS_BIN" live production-mutation-cancel-risk-gate \
    --run-id v180-production-mutation-cancel-risk-gate \
    --cancel-request-preview "$1" \
    --expected-symbol "$2" \
    --expected-account-label "$3" \
    --output "$4" \
    --allow-production-mutation-cancel-risk-gate \
    --confirm-single-v16-mutation-candidate-lineage \
    --confirm-cancel-request-preview-ready \
    --confirm-orphan-risk-halted \
    --confirm-known-order-identifier-only \
    --confirm-symbol-account-scope \
    --confirm-owner-approval-required \
    --confirm-no-cancel-all-or-bulk \
    --confirm-no-retry \
    --confirm-no-cancel \
    --confirm-no-network \
    --confirm-no-remediation \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-secret-persistence
}

run_gate_missing_flags() {
  "$NAUTILUS_BIN" live production-mutation-cancel-risk-gate \
    --run-id v180-production-mutation-cancel-risk-gate \
    --cancel-request-preview "$1" \
    --expected-symbol BTCUSDT \
    --expected-account-label prod-account-redacted \
    --output "$2"
}

run_classifier "$FIXTURE_DIR/open-orphan-mapper.json" "$OUTPUT_DIR/open-orphan-classifier.json" >/dev/null
run_orphan_detector "$OUTPUT_DIR/open-orphan-classifier.json" "$OUTPUT_DIR/open-orphan-detector.json" >/dev/null
run_preview "$OUTPUT_DIR/open-orphan-detector.json" "$OUTPUT_DIR/open-orphan-preview.json" >/dev/null

python3 - "$OUTPUT_DIR/open-orphan-preview.json" "$OUTPUT_DIR/forbidden-preview.json" <<'PY'
import json
import sys
from pathlib import Path

source = Path(sys.argv[1])
target = Path(sys.argv[2])
payload = json.loads(source.read_text())
for field in [
    "cancel_all_requested",
    "bulk_cancel_requested",
    "retry_requested",
    "replace_or_amend_requested",
    "flatten_requested",
    "dashboard_cancel_requested",
]:
    payload[field] = True
target.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY

run_gate \
  "$OUTPUT_DIR/open-orphan-preview.json" \
  BTCUSDT \
  prod-account-redacted \
  "$OUTPUT_DIR/open-orphan-risk-gate.json" >/dev/null
run_gate_missing_flags \
  "$OUTPUT_DIR/open-orphan-preview.json" \
  "$OUTPUT_DIR/missing-flags-risk-gate.json" >/dev/null
run_gate \
  "$OUTPUT_DIR/forbidden-preview.json" \
  ETHUSDT \
  other-account \
  "$OUTPUT_DIR/forbidden-risk-gate.json" >/dev/null

python3 - "$OUTPUT_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])

def load(name):
    return json.loads((root / name).read_text())

def assert_false_fields(artifact):
    for field in [
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
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "api_key_header_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
        "response_body_recorded",
        "response_headers_recorded",
    ]:
        assert artifact[field] is False, field
    assert artifact["cancel_requests_sent"] == 0

def assert_source_ref(artifact):
    ref = artifact["cancel_request_preview_ref"]
    assert ref["schema_version"] == "ntpro.v180_cancel_request_preview.v1"
    assert ref["hash"].startswith("fnv1a64:")
    assert ref["sha256"].startswith("sha256:")
    assert len(ref["sha256"]) == 71
    assert ref["bytes"] > 0
    assert ref["source_command"] == "nautilus live production-mutation-cancel-request-preview"
    assert ref["source_commit"] != "unknown"
    assert ref["source_release_tag"]

ready = load("open-orphan-risk-gate.json")
assert ready["schema_version"] == "ntpro.v180_cancel_risk_gate.v1"
assert ready["artifact_type"] == "cancel_risk_gate"
assert ready["status"] == "ready_cancel_risk_gate"
assert ready["cancel_request_preview_ready"] is True
assert ready["cancel_risk_gate_ready"] is True
assert ready["orphan_risk_detected"] is True
assert ready["risk_halted"] is True
assert ready["manual_review_required"] is True
assert ready["new_orders_blocked"] is True
assert ready["lineage_scope"] == "single_v16_mutation_candidate"
assert ready["order_identifier_known"] is True
assert ready["symbol_matches_lineage"] is True
assert ready["account_matches_lineage"] is True
assert ready["owner_approval_required"] is True
assert ready["owner_approval_lifecycle_recorded"] is False
assert ready["candidate_count"] == 1
assert ready["multi_order_cancel_requested"] is False
assert ready["cancel_all_requested"] is False
assert ready["bulk_cancel_requested"] is False
assert ready["strategy_driven_cancel_requested"] is False
assert ready["retry_requested"] is False
assert ready["replace_or_amend_requested"] is False
assert ready["flatten_requested"] is False
assert ready["dashboard_cancel_requested"] is False
assert ready["source_artifact_issues"] == []
assert ready["missing_cli_flags"] == []
assert_source_ref(ready)
assert_false_fields(ready)

missing = load("missing-flags-risk-gate.json")
assert missing["status"] == "blocked_missing_gate"
assert missing["cancel_request_preview_ready"] is True
assert missing["cancel_risk_gate_ready"] is False
assert "--allow-production-mutation-cancel-risk-gate" in missing["missing_cli_flags"]
assert "--confirm-symbol-account-scope" in missing["missing_cli_flags"]
assert missing["source_artifact_issues"] == []
assert_false_fields(missing)

forbidden = load("forbidden-risk-gate.json")
assert forbidden["status"] == "blocked_source_artifact"
assert forbidden["cancel_risk_gate_ready"] is False
assert forbidden["symbol_matches_lineage"] is False
assert forbidden["account_matches_lineage"] is False
for field in [
    "cancel_all_requested",
    "bulk_cancel_requested",
    "retry_requested",
    "replace_or_amend_requested",
    "flatten_requested",
    "dashboard_cancel_requested",
]:
    assert forbidden[field] is True, field
for issue in [
    "symbol_mismatch",
    "account_label_mismatch",
    "cancel_request_preview_cancel_all_requested_true",
    "cancel_request_preview_bulk_cancel_requested_true",
    "cancel_request_preview_retry_requested_true",
    "cancel_request_preview_replace_or_amend_requested_true",
    "cancel_request_preview_flatten_requested_true",
    "cancel_request_preview_dashboard_cancel_requested_true",
]:
    assert issue in forbidden["source_artifact_issues"], issue
assert forbidden["missing_cli_flags"] == []
assert_false_fields(forbidden)

for output in root.glob("*.json"):
    body = output.read_text()
    for token in [
        "X-MBX-APIKEY",
        "signature=",
        "apiSecret",
        "signedQuery=",
        "signedUrl=",
    ]:
        assert token not in body, f"{output.name} contains {token}"
for output in ["open-orphan-risk-gate.json", "missing-flags-risk-gate.json", "forbidden-risk-gate.json"]:
    body = (root / output).read_text()
    assert "123456789" not in body, output
    assert "owner-approved-v160-single-shot" not in body, output
PY

echo "verify_v18_cancel_risk_gate PASS root=$GATE_ROOT"
