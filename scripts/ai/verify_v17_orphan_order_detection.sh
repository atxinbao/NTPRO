#!/usr/bin/env bash
set -euo pipefail

# V170-004: v0.17 orphan order detection.
# This verifier stays local/offline. It detects open/orphan order risk from
# reconciliation classifier artifacts and proves risk cases halt new orders
# without retry, cancel, remediation, or Dashboard order controls.

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

GATE_ROOT="${NTPRO_V17_ORPHAN_ORDER_DETECTION_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v17-orphan-detector.XXXXXX")}"
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

def classifier(
    outcome,
    request_sent,
    exchange_status,
    exchange_state,
    order_found,
    open_observed,
    terminal,
    manual_review,
    new_orders_blocked,
    restart_readable=True,
):
    payload = {
        "schema_version": "ntpro.v170_production_mutation_reconciliation_classifier.v1",
        "run_id": "v170-production-mutation-reconciliation-classifier",
        "order_lineage_id": "lineage-v160-single-shot",
        "artifact_type": "production_mutation_reconciliation_classifier",
        "status": "ready_reconciliation_classified",
        "mode": "single_mutation_candidate_reconciliation_classifier",
        "capability": "Production Reconciliation And Orphan Recovery Evidence",
        "capability_expansion_from_v16": "reconciliation_evidence_only",
        "lineage_scope": "single_v16_mutation_candidate",
        "default_fail_closed": True,
        "owner_gated_readback_required": True,
        "exchange_readback_mapped": True,
        "reconciliation_classified": True,
        "orphan_risk_detected": False,
        "local_request_sent": request_sent,
        "exchange_order_status": exchange_status,
        "exchange_order_state": exchange_state,
        "open_order_observed": open_observed,
        "terminal_state_observed": terminal,
        "order_found": order_found,
        "reconciliation_outcome": outcome,
        "source_artifact_issues": [],
        "missing_cli_flags": [],
        "manual_review_required": manual_review,
        "new_orders_blocked": new_orders_blocked,
        "restart_readable": restart_readable,
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
    return payload

cases = {
    "clean-terminal": classifier("local_sent_exchange_filled", True, "FILLED", "filled", True, False, True, False, False),
    "open-orphan": classifier("local_sent_exchange_new", True, "NEW", "open", True, True, False, True, True),
    "local-missing-exchange-seen": classifier("local_no_send_exchange_order_seen", False, "NEW", "open", True, False, False, True, True),
    "readback-failure": classifier("readback_failed", True, "MALFORMED", "malformed", True, False, False, True, True),
    "stale-ledger-restart": classifier("local_sent_exchange_filled", True, "FILLED", "filled", True, False, True, False, False, False),
}

for name, payload in cases.items():
    write(root / f"{name}.json", payload)
PY

run_detector() {
  local classifier="$1"
  local output="$2"
  shift 2
  "$NAUTILUS_BIN" live production-mutation-orphan-order-detector \
    --run-id v170-production-mutation-orphan-order-detector \
    --reconciliation-classifier "$classifier" \
    --output "$output" \
    "$@"
}

run_ready_detector() {
  run_detector "$1" "$2" \
    --allow-production-mutation-orphan-order-detector \
    --confirm-single-v16-mutation-candidate-lineage \
    --confirm-read-only-reconciliation-scope \
    --confirm-no-retry \
    --confirm-no-cancel \
    --confirm-no-remediation \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-secret-persistence
}

run_detector \
  "$FIXTURE_DIR/open-orphan.json" \
  "$OUTPUT_DIR/missing-flags-orphan-detector.json" >/dev/null

for name in clean-terminal open-orphan local-missing-exchange-seen readback-failure stale-ledger-restart; do
  run_ready_detector \
    "$FIXTURE_DIR/$name.json" \
    "$OUTPUT_DIR/$name.json" >/dev/null
done

python3 - "$OUTPUT_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])

missing_flags = json.loads((root / "missing-flags-orphan-detector.json").read_text())
assert missing_flags["schema_version"] == "ntpro.v170_production_mutation_orphan_order_detector.v1"
assert missing_flags["status"] == "blocked_missing_gate"
assert missing_flags["orphan_detection_completed"] is False
assert missing_flags["orphan_risk_detected"] is True
assert missing_flags["risk_halted"] is True
assert "--allow-production-mutation-orphan-order-detector" in missing_flags["missing_cli_flags"]
assert "--confirm-no-cancel" in missing_flags["missing_cli_flags"]
assert missing_flags["retry_attempted"] is False
assert missing_flags["cancel_attempted"] is False
assert missing_flags["dashboard_order_controls_enabled"] is False

expected = {
    "clean-terminal": ("clean_terminal", False, False, False, False, False, True),
    "open-orphan": ("open_orphan_risk", True, True, True, True, False, False),
    "local-missing-exchange-seen": ("local_missing_exchange_seen", True, True, True, True, False, False),
    "readback-failure": ("readback_or_lineage_ambiguous", True, True, True, True, False, False),
    "stale-ledger-restart": ("stale_ledger_restart_required", True, True, True, True, True, True),
}
for name, (outcome, risk, halted, manual, blocked, stale_restart, local_terminal) in expected.items():
    artifact = json.loads((root / f"{name}.json").read_text())
    assert artifact["schema_version"] == "ntpro.v170_production_mutation_orphan_order_detector.v1"
    assert artifact["status"] == "ready_orphan_order_detection_completed"
    assert artifact["orphan_detection_completed"] is True
    assert artifact["orphan_detection_outcome"] == outcome
    assert artifact["orphan_risk_detected"] is risk
    assert artifact["risk_halted"] is halted
    assert artifact["manual_review_required"] is manual
    assert artifact["new_orders_blocked"] is blocked
    assert artifact["stale_ledger_restart_required"] is stale_restart
    assert artifact["local_terminal_state"] is local_terminal
    assert artifact["source_artifact_issues"] == []
    assert artifact["missing_cli_flags"] == []
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
        assert artifact[field] is False, f"{name} {field}"

for output in root.glob("*.json"):
    body = output.read_text()
    forbidden = [
        "X-MBX-APIKEY",
        "signature=",
        "apiSecret",
        "signedQuery=",
        "signedUrl=",
    ]
    for token in forbidden:
        assert token not in body, f"{output.name} contains {token}"
PY

echo "verify_v17_orphan_order_detection PASS root=$GATE_ROOT"
