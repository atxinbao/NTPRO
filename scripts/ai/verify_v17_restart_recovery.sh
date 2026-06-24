#!/usr/bin/env bash
set -euo pipefail

# V170-006: v0.17 restart recovery evidence.
# This verifier stays local/offline. It simulates a process restart by reusing
# an existing local order ledger artifact, then resumes readback,
# reconciliation, and orphan detection without duplicate submit, retry, cancel,
# remediation, or Dashboard order controls.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V17_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V17_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V17_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

GATE_ROOT="${NTPRO_V17_RESTART_RECOVERY_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v17-restart-recovery.XXXXXX")}"
LEDGER_ROOT="$GATE_ROOT/pre-restart-ledger"
RESTART_ROOT="$GATE_ROOT/post-restart"
FIXTURE_DIR="$RESTART_ROOT/fixtures"
OUTPUT_DIR="$RESTART_ROOT/command-output"
mkdir -p "$FIXTURE_DIR" "$OUTPUT_DIR"

NTPRO_V17_SKIP_BUILD=1 \
NTPRO_V17_NAUTILUS_BIN="$NAUTILUS_BIN" \
NTPRO_V17_LOCAL_ORDER_LEDGER_ROOT="$LEDGER_ROOT" \
  scripts/ai/verify_v17_local_order_ledger.sh >/dev/null

ORIGINAL_LEDGER="$LEDGER_ROOT/command-output/ready-local-order-ledger.json"
RECOVERED_LEDGER="$RESTART_ROOT/recovered-local-order-ledger.json"
if [[ ! -f "$ORIGINAL_LEDGER" ]]; then
  echo "restart recovery setup did not produce expected local ledger" >&2
  exit 1
fi
cp "$ORIGINAL_LEDGER" "$RECOVERED_LEDGER"

python3 - "$FIXTURE_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])

def write(path, payload):
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")

order_readback = {
    "schema_version": "ntpro.v170_redacted_binance_order_readback.v1",
    "artifact_type": "redacted_binance_order_readback",
    "status": "ready_redacted_order_readback_metadata",
    "endpoint": "order",
    "method": "GET",
    "path": "/api/v3/order",
    "order_found": True,
    "symbol": "BTCUSDT",
    "order_id": "123456789",
    "client_order_id": "owner-approved-v160-single-shot",
    "exchange_status": "NEW",
    "response_redacted": True,
}
open_orders_readback = {
    "schema_version": "ntpro.v170_redacted_binance_open_orders_readback.v1",
    "artifact_type": "redacted_binance_open_orders_readback",
    "status": "ready_redacted_open_orders_readback_metadata",
    "endpoint": "open_orders",
    "method": "GET",
    "path": "/api/v3/openOrders",
    "symbol": "BTCUSDT",
    "open_orders": [
        {
            "symbol": "BTCUSDT",
            "order_id": "123456789",
            "client_order_id": "owner-approved-v160-single-shot",
            "exchange_status": "NEW",
        }
    ],
    "response_redacted": True,
}
for payload in [order_readback, open_orders_readback]:
    for field in [
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "api_key_header_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
        "response_body_recorded",
        "response_headers_recorded",
        "request_sent",
        "network_attempted",
        "retry_attempted",
        "cancel_attempted",
        "replace_attempted",
        "amend_attempted",
        "flatten_attempted",
        "remediation_attempted",
        "dashboard_order_controls_enabled",
    ]:
        payload[field] = False

write(root / "order-readback-new.json", order_readback)
write(root / "open-orders-new.json", open_orders_readback)
PY

EXCHANGE_MAPPER="$OUTPUT_DIR/restarted-exchange-readback-mapper.json"
RECONCILIATION_CLASSIFIER="$OUTPUT_DIR/restarted-reconciliation-classifier.json"
ORPHAN_DETECTOR="$OUTPUT_DIR/restarted-orphan-detector.json"

"$NAUTILUS_BIN" live production-mutation-exchange-readback-mapper \
  --run-id v170-restart-recovery-exchange-readback-mapper \
  --local-order-ledger "$RECOVERED_LEDGER" \
  --order-readback "$FIXTURE_DIR/order-readback-new.json" \
  --open-orders-readback "$FIXTURE_DIR/open-orders-new.json" \
  --output "$EXCHANGE_MAPPER" \
  --allow-production-mutation-exchange-readback-mapper \
  --confirm-redacted-readback-metadata-only \
  --confirm-known-order-identifier-only \
  --confirm-read-only-reconciliation-scope \
  --confirm-no-network \
  --confirm-no-secret-persistence \
  --confirm-no-production-order-mutation \
  --confirm-no-retry \
  --confirm-no-cancel \
  --confirm-dashboard-order-controls-disabled >/dev/null

"$NAUTILUS_BIN" live production-mutation-reconciliation-classifier \
  --run-id v170-restart-recovery-reconciliation-classifier \
  --exchange-readback-mapper "$EXCHANGE_MAPPER" \
  --output "$RECONCILIATION_CLASSIFIER" \
  --allow-production-mutation-reconciliation-classifier \
  --confirm-single-v16-mutation-candidate-lineage \
  --confirm-read-only-reconciliation-scope \
  --confirm-no-retry \
  --confirm-no-cancel \
  --confirm-no-remediation \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-secret-persistence >/dev/null

"$NAUTILUS_BIN" live production-mutation-orphan-order-detector \
  --run-id v170-restart-recovery-orphan-detector \
  --reconciliation-classifier "$RECONCILIATION_CLASSIFIER" \
  --output "$ORPHAN_DETECTOR" \
  --allow-production-mutation-orphan-order-detector \
  --confirm-single-v16-mutation-candidate-lineage \
  --confirm-read-only-reconciliation-scope \
  --confirm-no-retry \
  --confirm-no-cancel \
  --confirm-no-remediation \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-secret-persistence >/dev/null

python3 - "$ORIGINAL_LEDGER" "$RECOVERED_LEDGER" "$EXCHANGE_MAPPER" "$RECONCILIATION_CLASSIFIER" "$ORPHAN_DETECTOR" <<'PY'
import json
import sys
from pathlib import Path

original_ledger = json.loads(Path(sys.argv[1]).read_text())
recovered_ledger = json.loads(Path(sys.argv[2]).read_text())
mapper = json.loads(Path(sys.argv[3]).read_text())
classifier = json.loads(Path(sys.argv[4]).read_text())
detector = json.loads(Path(sys.argv[5]).read_text())

assert recovered_ledger == original_ledger
assert recovered_ledger["schema_version"] == "ntpro.v170_production_mutation_local_order_ledger.v1"
assert recovered_ledger["status"] == "ready_local_order_ledger"
assert recovered_ledger["order_lineage_id"] == "lineage-v160-single-shot"
assert recovered_ledger["local_ledger_ready"] is True
assert recovered_ledger["restart_readable"] is True
assert recovered_ledger["current_local_state"] == "local_ledger_pending_exchange_reconciliation"

assert mapper["schema_version"] == "ntpro.v170_production_mutation_exchange_readback_mapper.v1"
assert mapper["status"] == "ready_exchange_readback_mapped"
assert mapper["local_ledger_ref"]["path"].endswith("recovered-local-order-ledger.json")
assert mapper["order_lineage_id"] == recovered_ledger["order_lineage_id"]
assert mapper["exchange_readback_mapped"] is True
assert mapper["exchange_order_state"] == "open"
assert mapper["open_order_observed"] is True
assert mapper["terminal_state_observed"] is False

assert classifier["schema_version"] == "ntpro.v170_production_mutation_reconciliation_classifier.v1"
assert classifier["status"] == "ready_reconciliation_classified"
assert classifier["reconciliation_classified"] is True
assert classifier["reconciliation_outcome"] == "local_no_send_exchange_order_seen"
assert classifier["manual_review_required"] is True
assert classifier["new_orders_blocked"] is True

assert detector["schema_version"] == "ntpro.v170_production_mutation_orphan_order_detector.v1"
assert detector["status"] == "ready_orphan_order_detection_completed"
assert detector["orphan_detection_completed"] is True
assert detector["orphan_detection_outcome"] == "failure_incident_risk_halt"
assert detector["orphan_risk_detected"] is True
assert detector["risk_halted"] is True
assert detector["manual_review_required"] is True
assert detector["new_orders_blocked"] is True
assert detector["stale_ledger_restart_required"] is False
assert detector["failure_mode"] == "readback-mismatch"
assert detector["failure_incident_outcome"] == "readback_mismatch_risk_halt"
assert detector["incident_risk_halted"] is True

for artifact in [recovered_ledger, mapper, classifier, detector]:
    for field in [
        "network_attempted",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
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
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
        "response_body_recorded",
        "response_headers_recorded",
    ]:
        assert artifact[field] is False, f"{artifact['artifact_type']} {field}"

assert recovered_ledger["production_order_submissions_attempted"] == 0
assert recovered_ledger["production_orders_submitted"] == 0
assert recovered_ledger["production_order_mutations_attempted"] == 0
PY

if grep -R "X-MBX-APIKEY\\|signature=\\|apiSecret\\|signedQuery=\\|signedUrl=" "$OUTPUT_DIR" "$RECOVERED_LEDGER" >/dev/null; then
  echo "restart recovery artifacts persisted forbidden secret or signed material" >&2
  exit 1
fi

echo "verify_v17_restart_recovery PASS root=$GATE_ROOT restart_readable=true duplicate_submit_attempted=false retry_attempted=false cancel_attempted=false risk_halted=true new_orders_blocked=true"
